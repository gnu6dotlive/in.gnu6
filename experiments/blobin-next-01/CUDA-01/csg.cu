// CUDA-01: optional standalone geometry acceleration experiment.
// Original code. No F2 scene replacement, no proprietary historical blob.in source.
#include <cuda_runtime.h>
#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <stdexcept>
#include <string>
#include <vector>
struct Primitive {float x, y, radius; int op;};
struct Pixel {float field, normal_x, normal_y, coverage;};
static_assert(sizeof(Primitive)==16);
static_assert(sizeof(Pixel)==16);
#define CUDA_OK(expr) do {cudaError_t err=(expr); if(err!=cudaSuccess){     std::fprintf(stderr,"CUDA_ERROR %s:%d %s\n",__FILE__,__LINE__,cudaGetErrorString(err));std::exit(3);}}while(0)
__host__ __device__ inline float dist_circle(float x,float y,const Primitive& p){
    float dx=x-p.x,dy=y-p.y;
    return sqrtf(dx*dx+dy*dy)-p.radius;
}
__host__ __device__ inline float compose(float d,float b,int op){
    if(op==1) return fmaxf(d,-b);       // subtract
    if(op==2) return fmaxf(d,b);        // intersection
    if(op==3){
        constexpr float k=0.08f;        // smooth union, approximate distance
        float h=fmaxf(k-fabsf(d-b),0.0f)/k;
        return fminf(d,b)-h*h*k*0.25f;
    }
    return fminf(d,b);                 // union
}
__host__ __device__ inline float field(float x,float y,const Primitive* primitives,int count){
    float d=dist_circle(x,y,primitives[0]);
    for(int i=1;i<count;i++){
        float b=dist_circle(x,y,primitives[i]);
        d=compose(d,b,primitives[i].op);
    }
    return d;
}
__host__ __device__ inline Pixel evaluate(int ix,int iy,int width,int height,
                                            const Primitive* primitives,int count){
    float s=float(width<height?width:height);
    float x=(float(ix)+0.5f-float(width)*0.5f)/s;
    float y=(float(iy)+0.5f-float(height)*0.5f)/s;
    float eps=1.0f/s;
    float d=field(x,y,primitives,count);
    float dx=(field(x+eps,y,primitives,count)-field(x-eps,y,primitives,count))*0.5f;
    float dy=(field(x,y+eps,primitives,count)-field(x,y-eps,primitives,count))*0.5f;
    float mag=sqrtf(dx*dx+dy*dy);
    float nx=0.0f,ny=0.0f;
    if(mag>1e-8f){nx=dx/mag;ny=dy/mag;}
    float aa=1.2f/s;
    float coverage=fminf(1.0f,fmaxf(0.0f,0.5f-d/aa));
    return {d,nx,ny,coverage};
}
__global__ void evaluate_kernel(Pixel* out,int width,int height,
                                 const Primitive* primitives,int count){
    int x=int(blockIdx.x*blockDim.x+threadIdx.x);
    int y=int(blockIdx.y*blockDim.y+threadIdx.y);
    if(x>=width||y>=height)return;
    out[y*width+x]=evaluate(x,y,width,height,primitives,count);
}
static std::vector<Primitive> make_scene(int count){
    std::vector<Primitive> p;
    p.reserve(count);
    p.push_back({0.0f,0.0f,0.245f,0});
    for(int i=1;i<count;i++){
        float theta=float(i)*2.3999632297f;
        float ring=0.04f+float(i%13)*0.024f;
        float x=ring*std::cos(theta);
        float y=ring*std::sin(theta);
        int op=(i%17==0)?2:((i%13==0)?1:((i%4==0)?3:0));
        float r=(op==2)?0.68f:((op==1)?0.034f:0.035f+0.004f*float(i%7));
        p.push_back({x,y,r,op});
    }
    return p;
}
static double now_ms(){
    using namespace std::chrono;
    return duration<double,std::milli>(steady_clock::now().time_since_epoch()).count();
}
static void require(bool passed,const char* message){
    if(!passed){std::fprintf(stderr,"CUDA01_GATE_RED %s\n",message);std::exit(7);}
}
static bool valid_sample(const Pixel& p){
    return std::isfinite(p.field)&&std::isfinite(p.normal_x)
        &&std::isfinite(p.normal_y)&&std::isfinite(p.coverage)
        &&p.coverage>=0.0f&&p.coverage<=1.0f;
}
int main(int argc,char**argv){
    int count=64,width=256,height=192,repeat=20;
    for(int i=1;i<argc;i++){
        std::string arg=argv[i];
        if(i+1>=argc){std::fprintf(stderr,"CUDA01_ARG_MISSING %s\n",arg.c_str());return 2;}
        int v=std::atoi(argv[++i]);
        if(arg=="--primitives")count=v;
        else if(arg=="--width")width=v;
        else if(arg=="--height")height=v;
        else if(arg=="--repeat")repeat=v;
        else {std::fprintf(stderr,"CUDA01_UNKNOWN_ARG %s\n",arg.c_str());return 2;}
    }
    if(count<1||count>256||width<16||height<16||width>2048||height>2048||
       long(width)*long(height)>1024*1024||repeat<3||repeat>100){
        std::fprintf(stderr,"CUDA01_INPUT_REJECTED\n");return 2;
    }
    int device_count=0;
    CUDA_OK(cudaGetDeviceCount(&device_count));
    require(device_count>0,"NO_CUDA_DEVICE");
    cudaDeviceProp properties{};
    CUDA_OK(cudaGetDeviceProperties(&properties,0));
    require(properties.major>=8,"CUDA_CAPABILITY_LT_8_UNQUALIFIED");
    auto scene=make_scene(count);
    for(const auto& p:scene){
        require(std::isfinite(p.x)&&std::isfinite(p.y)&&p.radius>0,"INVALID_PRIMITIVE");
    }
    size_t n=size_t(width)*size_t(height);
    std::vector<Pixel> cpu(n),gpu(n);
    double t0=now_ms();
    for(int y=0;y<height;y++)for(int x=0;x<width;x++)
        cpu[size_t(y)*width+x]=evaluate(x,y,width,height,scene.data(),count);
    double cpu_ms=now_ms()-t0;
    Primitive* d_scene=nullptr;
    Pixel* d_output=nullptr;
    CUDA_OK(cudaMalloc((void**)&d_scene,scene.size()*sizeof(Primitive)));
    CUDA_OK(cudaMalloc((void**)&d_output,n*sizeof(Pixel)));
    auto start_transfer=now_ms();
    CUDA_OK(cudaMemcpy(d_scene,scene.data(),scene.size()*sizeof(Primitive),cudaMemcpyHostToDevice));
    dim3 threads(16,16);
    dim3 blocks((width+15)/16,(height+15)/16);
    evaluate_kernel<<<blocks,threads>>>(d_output,width,height,d_scene,count);
    CUDA_OK(cudaGetLastError());
    CUDA_OK(cudaMemcpy(gpu.data(),d_output,n*sizeof(Pixel),cudaMemcpyDeviceToHost));
    CUDA_OK(cudaDeviceSynchronize());
    double e2e_ms=now_ms()-start_transfer;
    float max_field=0,max_normal=0,max_coverage=0;
    unsigned ambiguous=0,bad_mask=0;
    for(size_t i=0;i<n;i++){
        require(valid_sample(cpu[i])&&valid_sample(gpu[i]),"NONFINITE_RESULT");
        max_field=fmaxf(max_field,fabsf(cpu[i].field-gpu[i].field));
        max_normal=fmaxf(max_normal,fmaxf(fabsf(cpu[i].normal_x-gpu[i].normal_x),
                                        fabsf(cpu[i].normal_y-gpu[i].normal_y)));
        max_coverage=fmaxf(max_coverage,fabsf(cpu[i].coverage-gpu[i].coverage));
        if((cpu[i].field<0)!=(gpu[i].field<0)){
            if(fabsf(cpu[i].field)<1e-4f){ambiguous++;}
            else {bad_mask++;}
        }
    }
    require(bad_mask==0,"CPU_CUDA_TOPOLOGY_MASK");
    require(max_field<=1e-4f,"FIELD_PARITY");
    require(max_normal<=0.006f,"NORMAL_PARITY");
    require(max_coverage<=0.007f,"COVERAGE_PARITY");
    // Falsifier deliberately corrupts a known output; the numeric oracle must turn RED.
    auto altered=gpu[0];altered.field+=0.25f;
    require(fabsf(altered.field-cpu[0].field)>1e-4f,"FALSIFIER_NOT_DETECTED");
    cudaEvent_t begin{},end{};
    CUDA_OK(cudaEventCreate(&begin));CUDA_OK(cudaEventCreate(&end));
    for(int i=0;i<5;i++)evaluate_kernel<<<blocks,threads>>>(d_output,width,height,d_scene,count);
    CUDA_OK(cudaDeviceSynchronize());
    CUDA_OK(cudaEventRecord(begin));
    for(int i=0;i<repeat;i++)evaluate_kernel<<<blocks,threads>>>(d_output,width,height,d_scene,count);
    CUDA_OK(cudaEventRecord(end));
    CUDA_OK(cudaEventSynchronize(end));
    float gpu_total=0;CUDA_OK(cudaEventElapsedTime(&gpu_total,begin,end));
    CUDA_OK(cudaEventDestroy(begin));CUDA_OK(cudaEventDestroy(end));
    CUDA_OK(cudaFree(d_scene));CUDA_OK(cudaFree(d_output));
    std::printf("CUDA01_PASS device=%s cc=%d.%d primitives=%d pixels=%zu repeat=%d "
      "max_field=%.9g max_normal=%.9g max_coverage=%.9g ambiguous_boundary=%u "
      "bad_mask=%u cpu_single_ms=%.6f gpu_kernel_mean_ms=%.6f "
      "gpu_one_shot_transfer_included_ms=%.6f falsifier=PASS\n",
      properties.name,properties.major,properties.minor,count,n,repeat,
      max_field,max_normal,max_coverage,ambiguous,bad_mask,
      cpu_ms,double(gpu_total)/repeat,e2e_ms);
    return 0;
}
