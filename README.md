# rusted_os

## Building environment:
`rustup target add riscv64imac-unknown-none-elf`  
`cargo install cargo-binutils`  
`rustup component add llvm-tools-preview`  

## QEMU (ninja is required):
`wget https://download.qemu.org/qemu-6.2.0.tar.xz`  
`tar xvJf qemu-6.2.0.tar.xz`  
`cd qemu-6.2.0`  
`./configure --target-list=riscv32-softmmu,riscv64-softmmu`  
`make -j$(nproc)`  
`sudo make install`    
出现 ERROR: pkg-config binary 'pkg-config' not found 时，可以通过 sudo apt-get install pkg-config 安装；  
出现 ERROR: glib-2.48 gthread-2.0 is required to compile QEMU 时，可以通过 sudo apt-get install libglib2.0-dev 安装；  
出现 ERROR: pixman >= 0.21.8 not present 时，可以通过 sudo apt-get install libpixman-1-dev 安装。  

## Run:  
`make run`