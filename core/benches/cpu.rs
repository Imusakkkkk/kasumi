use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use oboromi_core::cpu::unicorn_interface::UnicornCPU;

const NOP: u32 = 0xd503201f;
const BRK: u32 = 0xd4200000;
// add x0, x0, #1
const ADD1: u32 = 0x91000400;
// sub x0, x0, #1
const SUB1: u32 = 0xd1000400;
// and x0, x0, x1
const AND: u32 = 0x8a010000;
// orr x0, x0, x1
const ORR: u32 = 0xaa010000;
// eor x0, x0, x1
const EOR: u32 = 0xca010000;
// b .+8 (skip next)
const B_FWD: u32 = 0x14000002;
// cbz x0, .+8
const CBZ: u32 = 0xb4000040;
// str w0, [x1]
const STR: u32 = 0xb9000020;
// ldr w0, [x1]
const LDR: u32 = 0xb9400020;

fn setup_cpu(n: usize, insn: u32) -> UnicornCPU {
    let cpu = UnicornCPU::new().expect("unicorn");
    let mut addr = 0u64;
    for _ in 0..n {
        cpu.write_u32(addr, insn);
        addr += 4;
    }
    cpu.write_u32(addr, BRK);
    cpu.set_pc(0);
    cpu.set_x(0, 0);
    cpu.set_x(1, 0x1000);
    cpu
}

fn run_checked(cpu: UnicornCPU, expected_x0: u64) {
    cpu.run();
    assert_eq!(cpu.get_x(0), expected_x0);
}

fn bench(c: &mut Criterion) {
    // 100 NOPs
    c.bench_function("unicorn_100_nops", |b| {
        b.iter_batched(|| setup_cpu(100, NOP), |cpu| cpu.run(), BatchSize::SmallInput)
    });
    // 100 ADDs: x0 ends 100
    c.bench_function("unicorn_100_adds", |b| {
        b.iter_batched(|| setup_cpu(100, ADD1), |cpu| {
            cpu.run();
            assert_eq!(cpu.get_x(0), 100);
        }, BatchSize::SmallInput)
    });
    // 100 SUBs
    c.bench_function("unicorn_100_subs", |b| {
        b.iter_batched(|| {
            let c = setup_cpu(100, SUB1);
            c.set_x(0, 100);
            c
        }, |cpu| {
            cpu.run();
            assert_eq!(cpu.get_x(0), 0);
        }, BatchSize::SmallInput)
    });
    // Logical AND/ORR/EOR
    for (name, insn) in [("and", AND), ("orr", ORR), ("eor", EOR)] {
        c.bench_function(&format!("unicorn_100_{}", name), |b| {
            b.iter_batched(|| {
                let cpu = setup_cpu(100, insn);
                cpu.set_x(0, 0xFF);
                cpu.set_x(1, 0x0F);
                cpu
            }, |cpu| {
                cpu.run();
            }, BatchSize::SmallInput)
        });
    }
    // BRK-only
    c.bench_function("unicorn_brk_only", |b| {
        b.iter_batched(|| {
            let cpu = UnicornCPU::new().expect("unicorn");
            cpu.write_u32(0, BRK);
            cpu.set_pc(0);
            cpu
        }, |cpu| cpu.run(), BatchSize::SmallInput)
    });
    // 1000 NOPs
    c.bench_function("unicorn_1000_nops", |b| {
        b.iter_batched(|| setup_cpu(1000, NOP), |cpu| cpu.run(), BatchSize::SmallInput)
    });

    // Branch unconditional: b .+8 repeated
    c.bench_function("unicorn_100_branches", |b| {
        b.iter_batched(|| setup_cpu(100, B_FWD), |cpu| cpu.run(), BatchSize::SmallInput)
    });

    // Conditional branch: cbz x0, .+8 (x0=0 -> taken)
    c.bench_function("unicorn_100_cond_branches", |b| {
        b.iter_batched(|| {
            let cpu = setup_cpu(100, CBZ);
            cpu.set_x(0, 0);
            cpu
        }, |cpu| cpu.run(), BatchSize::SmallInput)
    });

    // Memory: STR then LDR pattern (str w0,[x1]; ldr w0,[x1])
    c.bench_function("unicorn_mem_str_ldr", |b| {
        b.iter_batched(|| {
            let cpu = UnicornCPU::new().expect("unicorn");
            // write str/ldr pair 50 times
            let mut addr = 0u64;
            for _ in 0..50 {
                cpu.write_u32(addr, STR);
                cpu.write_u32(addr + 4, LDR);
                addr += 8;
            }
            cpu.write_u32(addr, BRK);
            cpu.set_pc(0);
            cpu.set_x(0, 0x1234);
            cpu.set_x(1, 0x1000);
            cpu
        }, |cpu| cpu.run(), BatchSize::SmallInput)
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);