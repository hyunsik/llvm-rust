use std::mem;
use std::ffi::CString;

use cbox::{CSemiBox, DisposeRef};
use ffi::prelude::{LLVMBuilderRef, LLVMValueRef};
use ffi::{core, LLVMBuilder, LLVMRealPredicate, LLVMIntPredicate};
use libc::{c_char, c_uint};

use context::Context;
use block::BasicBlock;
use ty::Type;
use value::{Function, Value, Predicate};
use phi::PhiNode;

static NULL_NAME:[c_char; 1] = [0];

/// This provides a uniform API for creating instructions and inserting them into a basic block.
pub struct Builder;
native_ref!(&Builder = LLVMBuilderRef);

macro_rules! bin_op (
  ($name:ident, $func:ident) => (
    pub fn $name(&self, left: &Value, right: &Value) -> &Value 
    {
      unsafe { core::$func(self.into(), left.into(), right.into(), NULL_NAME.as_ptr()) }.into()
    }
  );
  ($name:ident, $ifunc:ident, $ffunc:ident) => (
    pub fn $name(&self, left: &Value, right: &Value) -> &Value {
      let ty = left.get_type();
      unsafe {
        (if ty.is_integer() {
           core::$ifunc
         } else {
           core::$ffunc
         })(self.into(), left.into(), right.into(), NULL_NAME.as_ptr()).into()
      }
    }
  );
);

macro_rules! unary_op (
  ($name:ident, $func:ident) => (
    pub fn $name(&self, value: &Value) -> &Value {
      unsafe { core::$func(self.into(), value.into(), NULL_NAME.as_ptr() as *const c_char) }.into()
    }
  );
);

impl Builder 
{
  /// Create a new builder in the context given.
  pub fn new(context: &Context) -> CSemiBox<Builder> 
  {
    CSemiBox::new(unsafe { core::LLVMCreateBuilderInContext(context.into()) }.into())
  }
  
  pub fn get_insert_block(&self) -> &BasicBlock {
    unsafe { core::LLVMGetInsertBlock(self.into()) }.into()
  }
  
  /// Position the builder at `instr` within `block`.
  pub fn position_at(&self, block: &BasicBlock, instr: &Value) {
    unsafe { core::LLVMPositionBuilder(self.into(), block.into(), instr.into()) }
  }
  
  /// Position the builder at the end of `block`.
  pub fn position_at_end(&self, block: &BasicBlock) 
  {
    unsafe { core::LLVMPositionBuilderAtEnd(self.into(), block.into()) }
  }
  
  /// Build an instruction that returns from the function with void.
  pub fn create_ret_void(&self) -> &Value 
  {
    unsafe { core::LLVMBuildRetVoid(self.into()) }.into()
  }
  
  /// Build an instruction that returns from the function with `value`.
  pub fn create_ret(&self, value: &Value) -> &Value 
  {
    unsafe { core::LLVMBuildRet(self.into(), value.into()) }.into()
  }
  
  /// Build an instruction that allocates an array with the element type `elem` and the size `size`.
  ///
  /// The size of this array will be the size of `elem` times `size`.
  pub fn build_array_alloca(&self, elem: &Type, size: &Value) -> &Value 
  {
    unsafe { 
    	core::LLVMBuildArrayAlloca(self.into(), 
    														 elem.into(), 
    														 size.into(), 
    														 NULL_NAME.as_ptr() as *const c_char) 
    }.into()
  }
  
  /// Build an instruction that allocates a pointer to fit the size of `ty` then returns this pointer.
  ///
  /// Make sure to call `build_free` with the pointer value when you're done with it, or you're
  /// gonna have a bad time.
  pub fn create_alloca(&self, ty: &Type) -> &Value 
  {
    unsafe { 
    	core::LLVMBuildAlloca(self.into(), 
    												ty.into(), 
    												NULL_NAME.as_ptr() as *const c_char) 
    }.into()
  }
  
  /// Build an instruction that frees the `val`, which _MUST_ be a pointer that was returned
  /// from `build_alloca`.
  pub fn create_free(&self, val: &Value) -> &Value 
  {
    unsafe { core::LLVMBuildFree(self.into(), val.into()) }.into()
  }
  
  /// Build an instruction that store the value `val` in the pointer `ptr`.
  pub fn create_store(&self, val: &Value, ptr: &Value) -> &Value 
  {
    unsafe { core::LLVMBuildStore(self.into(), val.into(), ptr.into()) }.into()
  }
  
  /// Build an instruction that branches to the block `dest`.
  pub fn create_br(&self, dest: &BasicBlock) -> &Value 
  {
    unsafe { core::LLVMBuildBr(self.into(), dest.into()).into() }
  }
  
  /// Build an instruction that branches to `if_block` if `cond` evaluates to true, and `else_block` otherwise.
  pub fn create_cond_br(&self, cond: &Value, 
  	                    if_block: &BasicBlock, else_block: Option<&BasicBlock>) -> &Value 
  {
    unsafe { 
    	core::LLVMBuildCondBr(self.into(), 
    		                    cond.into(), 
    		                    if_block.into(), 
    		                    mem::transmute(else_block)).into() 
    }
  }
  
  /// Build an instruction that calls the function `func` with the arguments `args`.
  ///
  /// This will return the return value of the function.
  pub fn create_call(&self, func: &Function, args: &[&Value]) -> &Value 
  {
    unsafe {
        let call = core::LLVMBuildCall(self.into(), 
        	                             func.into(), args.as_ptr() as *mut LLVMValueRef, 
        	                             args.len() as c_uint, 
        	                             NULL_NAME.as_ptr());
        core::LLVMSetTailCall(call, 0);
        call.into()
    }
  }
  
  /// Build an instruction that calls the function `func` with the arguments `args`.
  ///
  /// This will return the return value of the function.
  pub fn create_tail_call(&self, func: &Function, args: &[&Value]) -> &Value 
  {
    unsafe {
        let call = core::LLVMBuildCall(self.into(), 
        	                             func.into(), args.as_ptr() as *mut LLVMValueRef, 
        	                             args.len() as c_uint, 
        	                             NULL_NAME.as_ptr());
        core::LLVMSetTailCall(call, 1);
        call.into()
    }
  }
  /// Build an instruction that yields to `true_val` if `cond` is equal to `1`, and `false_val` otherwise.
  pub fn create_select(&self, cond: &Value, true_val: &Value, false_val: &Value) -> &Value 
  {
    unsafe { 
    	core::LLVMBuildSelect(self.into(), 
    		                    cond.into(), 
    		                    true_val.into(), 
    		                    false_val.into(), 
    		                    NULL_NAME.as_ptr()).into() }
  }
  
  /// Build an instruction that casts a value into a certain type.
  pub fn create_bit_cast(&self, value: &Value, dest: &Type) -> &Value 
  {
    unsafe { core::LLVMBuildBitCast(
    		self.into(), 
    		value.into(), 
    		dest.into(), 
    		NULL_NAME.as_ptr()).into() 
    }
  }
  
  /// Build an instruction that inserts a value into an aggregate data value.
  pub fn create_insert_value(&self, agg: &Value, elem: &Value, index: usize) -> &Value 
  {
    unsafe { 
    	core::LLVMBuildInsertValue(self.into(), 
    		                         agg.into(), 
    		                         elem.into(), 
    		                         index as c_uint, 
    		                         NULL_NAME.as_ptr()).into() 
    }
  }
  
  /// Build an instruction that extracts a value from an aggregate type.
	pub fn create_extract_value(&self, agg: &Value, index: usize) -> &Value {
    unsafe { 
    	core::LLVMBuildExtractValue(self.into(), 
    		                          agg.into(), 
    		                          index as c_uint, 
    		                          NULL_NAME.as_ptr()).into() 
   	}
  }
  
  /// Build an instruction that computes the address of a subelement of an aggregate data structure.
  ///
  /// Basically type-safe pointer arithmetic.
  pub fn create_gep(&self, pointer: &Value, indices: &[&Value]) -> &Value 
  {
    unsafe { 
    	core::LLVMBuildInBoundsGEP(self.into(), 
    	                           pointer.into(), 
    	                           indices.as_ptr() as *mut LLVMValueRef, 
    	                           indices.len() as c_uint, 
    	                           NULL_NAME.as_ptr()).into() 
    }
  }
  
     /// Build an instruction to select a value depending on the predecessor of the current block.
  pub fn create_phi(&self, ty: &Type, name: &str) -> &PhiNode 
  {
	  unsafe { 
	  	core::LLVMBuildPhi(self.into(), ty.into(), CString::new(name).unwrap().as_ptr()) 
  	}.into()
  }  
  
  /// Build an instruction that runs whichever block matches the value, or `default` if none of them matched it.
  pub fn create_switch(&self, 
  	                   value: &Value, 
  	                   default: &BasicBlock, 
  	                   cases: &[(&Value, &BasicBlock)]) -> &Value 
  {
    unsafe {
      let switch = core::LLVMBuildSwitch(self.into(), 
      	                                 value.into(), 
      	                                 default.into(), 
      	                                 cases.len() as c_uint);
      for case in cases {
        core::LLVMAddCase(switch, case.0.into(), case.1.into());
      }
      switch.into()
    }
  }
  
  unary_op!{create_load, LLVMBuildLoad}
  unary_op!{create_neg, LLVMBuildNeg}
  unary_op!{create_not, LLVMBuildNot}
  
  bin_op!{create_add, LLVMBuildAdd, LLVMBuildFAdd}
  bin_op!{create_sub, LLVMBuildSub, LLVMBuildFSub}
  bin_op!{create_mul, LLVMBuildMul, LLVMBuildFMul}
  bin_op!{create_div, LLVMBuildSDiv, LLVMBuildFDiv}
  bin_op!{create_rem, LLVMBuildSRem, LLVMBuildFRem}
  bin_op!{create_shl, LLVMBuildShl}
  bin_op!{create_ashr, LLVMBuildAShr}
  bin_op!{create_and, LLVMBuildAnd}
  bin_op!{create_or, LLVMBuildOr}
  bin_op!{create_xor, LLVMBuildXor}
  
  fn create_cmp_internal(&self, l: &Value, r: &Value, 
  											 pred: Predicate, signed: bool) -> &Value {
  	let (lhs_ty, rhs_ty) = (l.get_type(), r.get_type());
    assert_eq!(lhs_ty, rhs_ty);
    
    if lhs_ty.is_integer() {
	    let p = match (pred, signed) {
	    	(Predicate::Equal, _)                  => LLVMIntPredicate::LLVMIntEQ,
	    	(Predicate::NotEqual, _)               => LLVMIntPredicate::LLVMIntNE,
	    	(Predicate::LessThan, true)            => LLVMIntPredicate::LLVMIntSLT,
	    	(Predicate::LessThan, false)           => LLVMIntPredicate::LLVMIntULT,
	    	(Predicate::LessThanOrEqual, true)     => LLVMIntPredicate::LLVMIntSLE,
	    	(Predicate::LessThanOrEqual, false)    => LLVMIntPredicate::LLVMIntULE,
	    	(Predicate::GreaterThan, true)         => LLVMIntPredicate::LLVMIntSGT,
	    	(Predicate::GreaterThan, false)        => LLVMIntPredicate::LLVMIntUGT,
	    	(Predicate::GreaterThanOrEqual, true)  => LLVMIntPredicate::LLVMIntSGE,
	    	(Predicate::GreaterThanOrEqual, false) => LLVMIntPredicate::LLVMIntUGE,
	    };
	    
	    unsafe {
		    core::LLVMBuildICmp(self.into(), 
	    		                  p, 
	     		                  l.into(), r.into(), 
	     		                  NULL_NAME.as_ptr())
	    }.into()
	     
    } else if lhs_ty.is_float() {
    	let p = match pred {
        Predicate::Equal => LLVMRealPredicate::LLVMRealOEQ,
        Predicate::NotEqual => LLVMRealPredicate::LLVMRealONE,
        Predicate::GreaterThan => LLVMRealPredicate::LLVMRealOGT,
        Predicate::GreaterThanOrEqual => LLVMRealPredicate::LLVMRealOGE,
        Predicate::LessThan => LLVMRealPredicate::LLVMRealOLT,
        Predicate::LessThanOrEqual => LLVMRealPredicate::LLVMRealOLE
      };
    	
   	  unsafe { 
      	core::LLVMBuildFCmp(self.into(), 
      		                 p, l.into(), r.into(), 
      		                 NULL_NAME.as_ptr()) 
      }.into()
    	 
    } else {
      panic!("expected numbers, got {:?}", lhs_ty)
    }
  } 
  
  /// Build an instruction to compare two values with the predicate given.
  pub fn create_cmp(&self, l: &Value, r: &Value, pred: Predicate) -> &Value 
  {
    self.create_cmp_internal(l, r, pred, true)
  }
  
  /// Build an instruction to compare two values with the predicate given.
  pub fn create_ucmp(&self, l: &Value, r: &Value, pred: Predicate) -> &Value 
  {
    self.create_cmp_internal(l, r, pred, false)
  }
}

impl DisposeRef for Builder {
  type RefTo = LLVMBuilder;
  #[inline(always)]
  unsafe fn dispose(ptr: LLVMBuilderRef) {
      core::LLVMDisposeBuilder(ptr)
  }
}
