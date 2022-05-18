# rusted_os

## Environment
### Rust & Cargo
```rust
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils
rustup component add llvm-tools-preview
```
### QEMU
```bash
sudo apt install pkg-config libglib2.0-dev libpixman-1-dev
wget https://download.qemu.org/qemu-6.2.0.tar.xz  
tar xvJf qemu-6.2.0.tar.xz  
cd qemu-6.2.0  
./configure --target-list=riscv64-softmmu,riscv64-linux-user  
make -j$(nproc)  
sudo make install    
```
---
## Run
### Build user lib
```bash
cd user
make build
```

### Run
```bash
cd os
make run
```
### Force quit
<kbd>Ctrl + a</kbd> + <kbd>x</kbd>

### Test
[About test](https://cvi3z1pqli.feishu.cn/wiki/wikcnWwUq3Okr3Rp34s5Fm1WnQf)