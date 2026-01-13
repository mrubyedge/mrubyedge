use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mrubyedge::yamrb::vm::VM;

fn bench_hash_operations(c: &mut Criterion) {
    let ruby_code = r#"
    def hash_ops
      h = {}

      1000.times do |i|
        j = i % 5
        h["k_hello_#{j}"] = "v_hello"
      end

      1000.times do |i|
        j = i % 5
        h["k_hello_#{j}"]
      end
    end

    hash_ops
    "#;

    let bin = unsafe {
        let mut context = mruby_compiler2_sys::MRubyCompiler2Context::new();
        context.compile(ruby_code).unwrap()
    };

    c.bench_function("Hash operations (1000 inserts + 1000 gets)", |b| {
        b.iter(|| {
            let mut rite = mrubyedge::rite::load(&bin).unwrap();
            let mut vm = VM::open(&mut rite);
            black_box(vm.run().expect("VM run failed"));
        })
    });
}

criterion_group!(benches, bench_hash_operations);
criterion_main!(benches);
