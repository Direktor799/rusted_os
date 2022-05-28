cd user
make build
cd ../fs_tool
./target/debug/fs_tool -s ../user/target/riscv64gc-unknown-none-elf/release/ -t ../os/ -b 8192
cd ../os
make run