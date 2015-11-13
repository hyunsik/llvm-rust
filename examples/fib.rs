extern crate llvm;
use llvm::*;
use llvm::Attribute::*;
fn main() {
    let ctx = Context::new();
    let module = Module::new("simple", &ctx);
    let func = module.add_function("fib", Type::get::<fn(u64) -> u64>(&ctx));
    func.add_attributes(&[NoUnwind, ReadNone]);
    let value = &func[0];
    
    let entry = func.append("entry");
    let on_zero = func.append("on_zero");
    let on_one = func.append("on_one");
    let default = func.append("default");
    let builder = Builder::new(&ctx);
    let zero = 0u64.compile(&ctx);
    let one = 1u64.compile(&ctx);
    
    builder.position_at_end(entry);
    builder.create_switch(value, default, &[
        (zero, on_zero),
        (one, on_one)
    ]);
    
    builder.position_at_end(on_zero);
    builder.create_ret(zero);
    builder.position_at_end(on_one);
    builder.create_ret(one);
    builder.position_at_end(default);
    let two = 2u64.compile(&ctx);
    let a = builder.create_sub(value, one);
    let b = builder.create_sub(value, two);
    let fa = builder.create_tail_call(func, &[a]);
    let fb = builder.create_tail_call(func, &[b]);
    builder.create_ret(builder.create_add(fa, fb));
    
    module.verify().unwrap();
    
    let ee = JitEngine::new(&module, JitOptions {opt_level: 0}).unwrap();
    ee.with_function(func, |fib: extern fn(u64) -> u64| {
        for i in 0..10 {
            println!("fib {} = {}", i, fib(i))
        }
    });
}
