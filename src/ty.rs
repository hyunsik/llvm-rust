use std::{fmt, mem};
use std::iter::Iterator;
use std::ops::Deref;

use ffi::prelude::LLVMTypeRef;
use ffi::{core, target, LLVMTypeKind};
use libc::{c_int, c_uint};

use compile::Compile;
use context::{Context, GetContext};
use target::TargetData;
use util::{self, CastFrom};


/// Defines how a value should be laid out in memory.
pub struct Type;
native_ref!(&Type = LLVMTypeRef);
get_context!(Type, LLVMGetTypeContext);
impl_display!(Type, LLVMPrintTypeToString);

impl Type 
{
  #[inline(always)]
  /// Get the type given as an LLVM type descriptor in the context given.
  pub fn get<'a, T>(ctx:&'a Context) -> &'a Type where T:Compile<'a> 
  {
      T::get_type(ctx)
  }
  
  #[inline(always)]
  /// Return the type kind for the type.
  pub fn kind(&self) -> LLVMTypeKind 
  {
  	unsafe { core::LLVMGetTypeKind(self.into()) }
  }
  
  #[inline(always)]
  pub fn void_ty<'a>(ctx: &'a Context) -> &'a Type 
  {
  	unsafe { core::LLVMVoidTypeInContext(ctx.into()) }.into()
  }
  
  #[inline(always)]
  pub fn bool_ty<'a>(ctx: &'a Context) -> &'a Type 
  {
  	Type::i8_ty(ctx)
  }
  
  #[inline(always)]
  pub fn i8_ty<'a>(ctx: &'a Context) -> &'a Type 
  {
  	unsafe { core::LLVMInt8TypeInContext(ctx.into()) }.into()
  }
  
  #[inline(always)]
  pub fn i16_ty<'a>(ctx: &'a Context) -> &'a Type 
  {
    unsafe { core::LLVMInt16TypeInContext(ctx.into()) }.into()
  }
  
  #[inline(always)]
  pub fn i32_ty<'a>(ctx: &'a Context) -> &'a Type 
  {
    unsafe { core::LLVMInt32TypeInContext(ctx.into()) }.into()
  }

	#[inline(always)]  	
  pub fn i64_ty<'a>(ctx: &'a Context) -> &'a Type 
  {
    unsafe { core::LLVMInt64TypeInContext(ctx.into()) }.into()
  }
  
  #[inline(always)]
  pub fn f32_ty<'a>(ctx: &'a Context) -> &'a Type 
  {
    unsafe { core::LLVMFloatTypeInContext(ctx.into()) }.into()
  }
  
  #[inline(always)]
  pub fn f64_ty<'a>(ctx: &'a Context) -> &'a Type 
  {
    unsafe { core::LLVMDoubleTypeInContext(ctx.into()) }.into()
  }
  
	/// Make a new array with the length given.
	#[inline(always)]
  pub fn array_ty<'a>(element: &'a Type, length: usize) -> &'a Type 
  {
    unsafe { core::LLVMArrayType(element.into(), length as c_uint) }.into()
  }
  
  /// Make a new function signature with the return type and arguments given.
  #[inline(always)]
  pub fn function_ty<'a>(ret: &'a Type, args: &[&'a Type]) -> &'a FunctionType 
  {
    unsafe { 
    	core::LLVMFunctionType(ret.into(), 
    		                     args.as_ptr() as *mut LLVMTypeRef, 
     		                     args.len() as c_uint, 0) 
   	}.into()
  }
  
  /// Make a new vector with the length given.
  #[inline(always)]
  pub fn vector_ty<'a>(element: &'a Type, length: usize) -> &'a Type 
  {
    unsafe { core::LLVMVectorType(element.into(), length as c_uint) }.into()
  }
  
  /// Make a new pointer with the given element type.
  #[inline(always)]
  pub fn pointer_ty<'a>(elem: &'a Type) -> &'a Type 
  {
    unsafe { core::LLVMPointerType(elem.into(), 0 as c_uint) }.into()
  }
  
  /// Make a new structure type with the given types.
  #[inline(always)]
  pub fn struct_ty<'a>(elems: &[&'a Type], packed: bool) -> &'a Type 
  {
    unsafe { 
    	core::LLVMStructType(elems.as_ptr() as *mut LLVMTypeRef, 
    		                   elems.len() as c_uint, 
    		                   packed as c_int) 
   	}.into()
  }
  
  /// Returns true if the size of the type is known at compile-time.
  ///
  /// This is equivalent to the type implementing `Sized` in Rust
  #[inline(always)]
  pub fn is_sized(&self) -> bool 
  {
    unsafe { core::LLVMTypeIsSized(self.into()) != 0 }
  }
  
  /// Returns true if this type is a function.
  #[inline(always)]
  pub fn is_function(&self) -> bool 
  {
    let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
    kind as c_uint == LLVMTypeKind::LLVMFunctionTypeKind as c_uint
  }
  
  /// Returns true if this type is a struct.
  #[inline(always)]
  pub fn is_struct(&self) -> bool 
  {
    let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
    kind as c_uint == LLVMTypeKind::LLVMStructTypeKind as c_uint
  }
  
  /// Returns true if this type is void.
  #[inline(always)]
  pub fn is_void(&self) -> bool 
  {
    let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
    kind as c_uint == LLVMTypeKind::LLVMVoidTypeKind as c_uint
  }
  
  /// Returns true if this type is a pointer.
  #[inline(always)]
  pub fn is_pointer(&self) -> bool 
  {
    let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
    kind as c_uint == LLVMTypeKind::LLVMPointerTypeKind as c_uint
  }
  
  /// Returns true if this type is an integer.
  #[inline(always)]
  pub fn is_integer(&self) -> bool 
  {
    let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
    kind as c_uint == LLVMTypeKind::LLVMIntegerTypeKind as c_uint
  }
  
  /// Returns true if this type is any floating-point number.
  #[inline(always)]
  pub fn is_float(&self) -> bool 
  {
    let kind = unsafe { core::LLVMGetTypeKind(self.into()) } as c_uint;
    kind == LLVMTypeKind::LLVMHalfTypeKind as c_uint ||
    kind == LLVMTypeKind::LLVMFloatTypeKind as c_uint ||
    kind == LLVMTypeKind::LLVMDoubleTypeKind as c_uint
  }
  
  /// Returns the size of the type in bytes.
  #[inline(always)]
  pub fn get_size(&self, target: &TargetData) -> usize 
  {
    unsafe { target::LLVMABISizeOfType(target.into(), self.into()) as usize }
  }
  
  /// Returns the element of this pointer type.
  #[inline(always)]
  pub fn get_element(&self) -> Option<&Type> 
  {
    unsafe { mem::transmute(core::LLVMGetElementType(self.into())) }
  }
}


/// A structure type, such as a tuple or struct.
pub struct StructType;
native_ref!(&StructType = LLVMTypeRef);
deref!(StructType, Type);
get_context!(StructType, LLVMGetTypeContext);
impl_display!(StructType, LLVMPrintTypeToString);

impl StructType 
{
  /// Make a new struct with the given fields and packed representation.
  pub fn new<'a>(context: &'a Context, fields: &[&'a Type], packed: bool) -> &'a StructType 
  {
    unsafe { 
    	core::LLVMStructTypeInContext(context.into(), 
    		                            fields.as_ptr() as *mut LLVMTypeRef, 
    		                            fields.len() as c_uint, 
     		                            packed as c_int) 
   	}.into()
  }
  
  /// Make a new named struct with the given fields and packed representation.
  pub fn new_named<'a>(context: &'a Context, name: &str, 
  	                   fields: &[&'a Type], packed: bool) -> &'a StructType 
  {
    util::with_cstr(name, |name| unsafe {
        let ty = core::LLVMStructCreateNamed(context.into(), name);
        core::LLVMStructSetBody(ty, 
         	                      fields.as_ptr() as *mut LLVMTypeRef, 
         	                      fields.len() as c_uint,
         	                      packed as c_int);
        ty.into()
    })
  }
  
  /// Returns the elements that make up this struct.
  pub fn get_elements(&self) -> Vec<&Type> 
  {
    unsafe {
      let size = core::LLVMCountStructElementTypes(self.into());
      let mut els:Vec<_> = (0..size).map(|_| mem::uninitialized()).collect();
      core::LLVMGetStructElementTypes(self.into(), els.as_mut_ptr() as *mut LLVMTypeRef);
        
      els
    }
  }
}


impl CastFrom for StructType 
{
  type From = Type;
  fn cast(ty: &Type) -> Option<&StructType> 
  {
    unsafe {
      let kind = core::LLVMGetTypeKind(ty.into());
      if kind as c_uint == LLVMTypeKind::LLVMStructTypeKind as c_uint {
        mem::transmute(ty)
      } else {
        None
      }
    }
  }
}


/// A function signature type.
pub struct FunctionType;
native_ref!(&FunctionType = LLVMTypeRef);
deref!(FunctionType, Type);
get_context!(FunctionType, LLVMGetTypeContext);
impl_display!(FunctionType, LLVMPrintTypeToString);

impl FunctionType 
{
  /// Returns the number of parameters this signature takes.
  pub fn num_params(&self) -> usize 
  {
    unsafe { core::LLVMCountParamTypes(self.into()) as usize }
  }

  /// Returns a vector of this signature's parameters' types.
  pub fn get_params(&self) -> Vec<&Type> 
  {
  	unsafe {
    	let count = core::LLVMCountParamTypes(self.into());
      let mut types:Vec<_> = (0..count).map(|_| mem::uninitialized()).collect();
      core::LLVMGetParamTypes(self.into(), types.as_mut_ptr() as *mut LLVMTypeRef);
      types
    }
  }
  
  /// Returns the type that this function returns.
  pub fn get_return(&self) -> &Type {
    unsafe { core::LLVMGetReturnType(self.into()).into() }
  }
}

impl CastFrom for FunctionType {
  type From = Type;
  fn cast(mut ty: &Type) -> Option<&FunctionType> {
    unsafe {
      use libc::c_uint;
      while let Some(elem) = ty.get_element() {
	      ty = elem;
      }
      
      let kind = core::LLVMGetTypeKind(ty.into());
      if kind as c_uint == LLVMTypeKind::LLVMFunctionTypeKind as c_uint {
      	mem::transmute(ty)
      } else {
      	None
      }
    }
  }
}


#[cfg(test)]
mod tests {
	use context::Context;
	use super::*;
	
	#[test]
	pub fn test_types() 
	{
		let ctx    = Context::new();
		assert_eq!("i8",     format!("{}", Type::i8_ty(&ctx)));
		assert_eq!("i16",    format!("{}", Type::i16_ty(&ctx)));
		assert_eq!("i32",    format!("{}", Type::i32_ty(&ctx)));
		assert_eq!("i64",    format!("{}", Type::i64_ty(&ctx)));
		assert_eq!("float",  format!("{}", Type::f32_ty(&ctx)));
		assert_eq!("double", format!("{}", Type::f64_ty(&ctx)));
		
		assert_eq!("[10 x double]",  format!("{}", Type::array_ty(&Type::f64_ty(&ctx), 10)));
	}
}
