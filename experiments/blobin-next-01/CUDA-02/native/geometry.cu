// CUDA-02 bounded optional provider ABI. This code is original and loads only on request.
// Geometry values only, never F2 epoch, identity, scene authority or mutation.
#include <cuda_runtime.h>
#include <stdint.h>
#include <math.h>
struct Primitive { float x, y, radius; uint32_t op; };
struct Sample { float distance, nx, ny, coverage; };
static_assert(sizeof(Primitive)==16,"ABI PRIMITIVE");
static_assert(sizeof(Sample)==16,"ABI SAMPLE");
__host__ __device__ inline float circle(float x,float y,Primitive p){
    float dx=x-p.x,dy=y-p.y;
    return sqrtf(dx*dx+dy*dy)-p.radius;
}
__host__ __device__ inline float field(float x,float y,const Primitive* p,uint32_t count){
    float d=circle(x,y,p[0]);
    for(uint32_t i=1;i<count;i++){
        float v=circle(x,y,p[i]);
        if(p[i].op==1){d=fmaxf(d,-v);}
        else if(p[i].op==2){d=fmaxf(d,v);}
        else if(p[i].op==3){
            float k=0.08f;
            float h=fmaxf(k-fabsf(d-v),0.0f)/k;
            d=fminf(d,v)-h*h*k*0.25f;
        }else{d=fminf(d,v);}
    }
    return d;
}
__global__ void compute(Sample* out,const Primitive* p,uint32_t width,uint32_t height,uint32_t count){
    uint32_t ix=blockIdx.x*blockDim.x+threadIdx.x;
    uint32_t iy=blockIdx.y*blockDim.y+threadIdx.y;
    if(ix>=width||iy>=height)return;
    float s=float(width<height?width:height);
    float x=(float(ix)+0.5f-float(width)*0.5f)/s;
    float y=(float(iy)+0.5f-float(height)*0.5f)/s;
    float eps=1.0f/s;
    float d=field(x,y,p,count);
    float dx=(field(x+eps,y,p,count)-field(x-eps,y,p,count))*0.5f;
    float dy=(field(x,y+eps,p,count)-field(x,y-eps,p,count))*0.5f;
    float mag=sqrtf(dx*dx+dy*dy);
    float nx=0.0f,ny=0.0f;
    if(mag>1e-8f){nx=dx/mag;ny=dy/mag;}
    float aa=1.2f/s;
    float cov=fminf(1.0f,fmaxf(0.0f,0.5f-d/aa));
    out[iy*width+ix]={d,nx,ny,cov};
}
// Error boundary: nonzero code means all output bytes are NOT qualified.
// No asynchronous lifetime or external GPU context escapes the ABI.
extern "C" int blobin_cuda02_evaluate(const Primitive* host,const uint32_t count,
                                      const uint32_t width,const uint32_t height,
                                      Sample* output,const uint64_t output_len){
    if(!host||!output||count<1||count>256||width<16||height<16||
       width>2048||height>2048||uint64_t(width)*height>1048576u||
       output_len!=uint64_t(width)*height)return 2;
    int available=0;
    if(cudaGetDeviceCount(&available)!=cudaSuccess||available==0)return 3;
    Primitive* d_primitive=nullptr;
    Sample* d_sample=nullptr;
    size_t primitive_bytes=size_t(count)*sizeof(Primitive);
    size_t output_bytes=size_t(output_len)*sizeof(Sample);
    int result=0;
    if(cudaMalloc((void**)&d_primitive,primitive_bytes)!=cudaSuccess)return 4;
    if(cudaMalloc((void**)&d_sample,output_bytes)!=cudaSuccess){result=5;goto cleanup;}
    if(cudaMemcpy(d_primitive,host,primitive_bytes,cudaMemcpyHostToDevice)!=cudaSuccess){result=6;goto cleanup;}
    {
        dim3 threads(16,16);
        dim3 grid((width+15)/16,(height+15)/16);
        compute<<<grid,threads>>>(d_sample,d_primitive,width,height,count);
    }
    if(cudaGetLastError()!=cudaSuccess){result=7;goto cleanup;}
    if(cudaMemcpy(output,d_sample,output_bytes,cudaMemcpyDeviceToHost)!=cudaSuccess){result=8;goto cleanup;}
    if(cudaDeviceSynchronize()!=cudaSuccess){result=9;goto cleanup;}
cleanup:
    if(d_sample)cudaFree(d_sample);
    if(d_primitive)cudaFree(d_primitive);
    return result;
}
