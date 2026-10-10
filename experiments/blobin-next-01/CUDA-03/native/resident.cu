// CUDA-03: persistent geometry buffers using unchanged CUDA-02 numerical kernel.
// Original lab wrapper; includes the qualified baseline kernel *by reference*.
#include "../../CUDA-02/native/geometry.cu"
#include <stdint.h>
#include <math.h>
#include <new>
struct Resident {
    Primitive* d_primitive;
    Sample* d_sample;
    uint32_t count,width,height;
    cudaEvent_t start,stop;
    bool events;
};
static bool valid(const Primitive* p,uint32_t count,uint32_t w,uint32_t h,uint64_t n){
    if(!p||count<1||count>256||w<16||h<16||w>2048||h>2048||
       uint64_t(w)*h>1048576u||n!=uint64_t(w)*h) return false;
    if(p[0].op!=0)return false;
    for(uint32_t i=0;i<count;i++){
        if(!isfinite(p[i].x)||!isfinite(p[i].y)||!isfinite(p[i].radius)||
           !(p[i].radius>0.0f)||p[i].op>3)return false;
    }
    return true;
}
extern "C" void blobin_cuda03_destroy(void* ptr);
extern "C" void* blobin_cuda03_create(const Primitive* p,uint32_t count,uint32_t width,uint32_t height,uint64_t n) {
    if(!valid(p,count,width,height,n))return nullptr;
    int devices=0;
    if(cudaGetDeviceCount(&devices)!=cudaSuccess||devices<1)return nullptr;
    auto* state=new(std::nothrow) Resident{nullptr,nullptr,count,width,height,{},{},false};
    if(!state)return nullptr;
    const auto pbytes=size_t(count)*sizeof(Primitive);
    const auto sbytes=size_t(n)*sizeof(Sample);
    if(cudaMalloc((void**)&state->d_primitive,pbytes)!=cudaSuccess) goto failed;
    if(cudaMalloc((void**)&state->d_sample,sbytes)!=cudaSuccess) goto failed;
    if(cudaMemcpy(state->d_primitive,p,pbytes,cudaMemcpyHostToDevice)!=cudaSuccess) goto failed;
    if(cudaEventCreate(&state->start)!=cudaSuccess) goto failed;
    if(cudaEventCreate(&state->stop)!=cudaSuccess){
        cudaEventDestroy(state->start);
        goto failed;
    }
    state->events=true;
    return state;
failed:
    blobin_cuda03_destroy(state);
    return nullptr;
}
extern "C" int blobin_cuda03_update(void* ptr,const Primitive* p,uint32_t count){
    if(!ptr)return 2;
    auto* state=static_cast<Resident*>(ptr);
    if(count!=state->count||!valid(p,count,state->width,state->height,uint64_t(state->width)*state->height))return 2;
    return cudaMemcpy(state->d_primitive,p,size_t(count)*sizeof(Primitive),cudaMemcpyHostToDevice)==cudaSuccess?0:5;
}
extern "C" int blobin_cuda03_run(void* ptr,float* ms){
    if(!ptr||!ms)return 2;
    auto* state=static_cast<Resident*>(ptr);
    dim3 threads(16,16),grid((state->width+15)/16,(state->height+15)/16);
    if(cudaEventRecord(state->start)!=cudaSuccess)return 5;
    compute<<<grid,threads>>>(state->d_sample,state->d_primitive,state->width,state->height,state->count);
    if(cudaGetLastError()!=cudaSuccess)return 6;
    if(cudaEventRecord(state->stop)!=cudaSuccess)return 7;
    if(cudaEventSynchronize(state->stop)!=cudaSuccess)return 8;
    if(cudaEventElapsedTime(ms,state->start,state->stop)!=cudaSuccess)return 9;
    return 0;
}
extern "C" int blobin_cuda03_copy(void* ptr,Sample* out,uint64_t n){
    if(!ptr||!out)return 2;
    auto* state=static_cast<Resident*>(ptr);
    if(n!=uint64_t(state->width)*state->height)return 2;
    return cudaMemcpy(out,state->d_sample,size_t(n)*sizeof(Sample),cudaMemcpyDeviceToHost)==cudaSuccess?0:5;
}
extern "C" uint64_t blobin_cuda03_resident_bytes(void* ptr){
    if(!ptr)return 0;
    auto* state=static_cast<Resident*>(ptr);
    return uint64_t(state->count)*sizeof(Primitive)+uint64_t(state->width)*state->height*sizeof(Sample);
}
extern "C" void blobin_cuda03_destroy(void* ptr){
    if(!ptr)return;
    auto* state=static_cast<Resident*>(ptr);
    if(state->events){
        cudaEventDestroy(state->start);
        cudaEventDestroy(state->stop);
    }
    if(state->d_sample)cudaFree(state->d_sample);
    if(state->d_primitive)cudaFree(state->d_primitive);
    delete state;
}
