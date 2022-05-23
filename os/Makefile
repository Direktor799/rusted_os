TARGET	    := riscv64gc-unknown-none-elf
MODE		:= debug
KERNEL_FILE := target/$(TARGET)/$(MODE)/rusted_os
BIN_FILE	:= target/$(TARGET)/$(MODE)/kernel.bin
FS_IMG		:= fs.img

OBJDUMP	 := rust-objdump --arch-name=riscv64
OBJCOPY	 := rust-objcopy --binary-architecture=riscv64

.PHONY: doc kernel build clean qemu run

# 默认 build 为输出二进制文件
build: $(BIN_FILE) 

# 通过 Rust 文件中的注释生成 os 的文档
doc:
	@cargo doc --document-private-items --no-deps --open

# 编译 kernel
kernel:
	@cargo build

# 生成 kernel 的二进制文件
$(BIN_FILE): kernel
	@$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $@

# 查看反汇编结果
asm:
	@$(OBJDUMP) -d $(KERNEL_FILE) | less

# 清理编译出的文件
clean:
	@cargo clean

# 运行 QEMU
qemu:
	@qemu-system-riscv64 \
			-machine virt \
			-nographic \
			-bios default \
			-kernel $(BIN_FILE)\
			-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
			-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0\
			-device loader,file=$(BIN_FILE),addr=0x80200000

qemudbg: 
	@qemu-system-riscv64 \
			-machine virt \
			-nographic \
			-bios default \
			-kernel $(BIN_FILE)\
			-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
			-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0\
			-device loader,file=$(BIN_FILE),addr=0x80200000 \
			-s -S

# 一键运行
run: build qemu

debug: build qemudbg

test:
	@cargo test --no-run
	@$(OBJCOPY) target/riscv64gc-unknown-none-elf/debug/deps/rusted_os-de323c8ea6968b7e --strip-all -O binary target/riscv64gc-unknown-none-elf/debug/deps/test
	@qemu-system-riscv64 \
			-machine virt \
			-nographic \
			-bios default \
			-kernel target/riscv64gc-unknown-none-elf/debug/deps/test\
			-device loader,file=target/riscv64gc-unknown-none-elf/debug/deps/test,addr=0x80200000
