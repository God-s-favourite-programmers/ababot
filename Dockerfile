FROM nvidia/vulkan:1.3-470 as gpu

COPY ababot.bin /ababot
RUN rm /etc/apt/sources.list.d/cuda.list
RUN apt-key del 7fa2af80
RUN wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2004/x86_64/cuda-keyring_1.0-1_all.deb
RUN dpkg -i cuda-keyring_1.0-1_all.deb
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y --no-install-recommends tzdata && apt-get install libvulkan1 && apt-get install nvidia-driver-525 -y

CMD ["/ababot"]