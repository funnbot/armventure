RUSTFLAGS="-C debuginfo=full -C target-cpu=znver3 -A warnings" cargo build --release --lib
llvm-objdump --debug-file-directory=target/release --debug-vars=ascii --debug-vars-indent=4 --disassemble --x86-asm-syntax=intel --show-all-symbols --line-numbers --symbolize-operands --no-leading-addr --symbol-description --no-show-raw-insn --source --arch=x86-64 target/release/libarmventure.rlib | rustfilt > av.asm

# RUSTFLAGS="-C debuginfo=full -C target-cpu=znver3 -A warnings" cargo build --release
# llvm-objdump --debug-file-directory=target/release --debug-vars=ascii --debug-vars-indent=4 --disassemble --x86-asm-syntax=intel --show-all-symbols --line-numbers --symbolize-operands --no-leading-addr --symbol-description --no-show-raw-insn --source --arch=x86-64 target/release/armventure.exe | rustfilt > av.asm

# RUSTFLAGS="-C debuginfo=full -C target-cpu=znver3 -A warnings" cargo build
# llvm-objdump --debug-file-directory=target/debug --debug-vars=ascii --debug-vars-indent=4 --disassemble --x86-asm-syntax=intel --show-all-symbols --line-numbers --symbolize-operands --no-leading-addr --symbol-description --no-show-raw-insn --source --arch=x86-64 target/debug/libarmventure.rlib | rustfilt > av.asm