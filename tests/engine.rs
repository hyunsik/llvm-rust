extern crate libc;
extern crate llvm;

use libc::c_void;
use llvm::*;
use std::mem;

pub extern "C" fn test_func3(x: f64) -> f64 {
  x
}

#[test]
fn add_global_mapping() {
  let ctx = Context::new();
  let module = Module::new("test_func_find", &ctx); 
  module.verify().expect("verifying the module failed...");
 
  let ee = JitEngine::new(&module, JitOptions {opt_level: 0}).unwrap();
  
  let ret_ty    = Type::get::<f64>(&ctx);
  let param_tys = vec![Type::get::<f64>(&ctx)];
  let fn_ty     = Type::function_ty(ret_ty, &param_tys);
  let function  = module.add_function("test_func3", fn_ty);
  let fn_ptr: *const c_void = unsafe { mem::transmute(test_func3) };
  unsafe { ee.add_global_mapping(function, fn_ptr); }  
  
	let f: fn(f64) -> f64;
  f = match unsafe { ee.get_function_raw(function) } {
  	Some(f) => unsafe { mem::transmute(f)},
  	_       => panic!("get_function_raw: couldn't find test_func3")
  };
  
  assert_eq!(98.0f64, f(98.0f64));
}