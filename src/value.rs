use std::ffi::CString;
use std::{fmt, mem, ptr};
use std::ops::{Deref, Index};

use libc::{c_char, c_int, c_uint};
use ffi::core;
use ffi::prelude::LLVMValueRef;
use ffi::LLVMAttribute;
use ffi::core::{
	LLVMConstStringInContext,
	LLVMConstStructInContext,
	LLVMConstVector,
	
	LLVMGetValueName,
	LLVMSetValueName,
	
	LLVMGetElementType,
	LLVMGetEntryBasicBlock,
	LLVMAppendBasicBlockInContext,
	
	LLVMAddAttribute,
	LLVMGetAttribute,	
	LLVMRemoveAttribute,
  
  LLVMAddFunctionAttr,
  LLVMGetFunctionAttr,
  LLVMRemoveFunctionAttr,
  
  LLVMGetParam,
  LLVMCountParams,
  LLVMGetFirstParam,
  LLVMGetNextParam,
  
  LLVMGetInitializer,
  LLVMSetInitializer,
  
  LLVMIsAGlobalValue,
  LLVMGetUndef,
  LLVMTypeOf,
};

use block::BasicBlock;
use context::{Context, GetContext};
use util::{self, CastFrom};
use ty::{FunctionType, Type};

/// A typed value that can be used as an operand in instructions.
pub struct Value;
native_ref!(&Value = LLVMValueRef);
impl_display!(Value, LLVMPrintValueToString);

impl Value 
{    
  /// Create a new constant struct from the values given.
  pub fn new_struct<'a>(context: &'a Context, 
  	                    vals: &[&'a Value], packed: bool) -> &'a Value 
  {
    unsafe { 
    	LLVMConstStructInContext(context.into(), 
    	                         vals.as_ptr() as *mut LLVMValueRef, 
    		                       vals.len() as c_uint, 
    		                       packed as c_int) 
    }.into()
  }
  
  /// Create a new constant vector from the values given.
  pub fn new_vector<'a>(vals: &[&'a Value]) -> &'a Value 
  {
    unsafe { 
    	LLVMConstVector(vals.as_ptr() as *mut LLVMValueRef, 
    		              vals.len() as c_uint)
    }.into()
  }
  
  /// Create a new constant C string from the text given.
  pub fn new_string<'a>(context: &'a Context, 
  	                    text: &str, 
  	                    rust_style: bool) -> &'a Value 
  {
    unsafe {
      let ptr = text.as_ptr() as *const c_char;
      let len = text.len() as c_uint;
      LLVMConstStringInContext(context.into(), ptr, len, rust_style as c_int).into()
    }
  }
  
  /// Create a new constant undefined value of the given type.
  pub fn new_undef<'a>(ty: &'a Type) -> &'a Value 
  {
    unsafe { LLVMGetUndef(ty.into()) }.into()
  }
  
  /// Returns the name of this value, or `None` if it lacks a name
  pub fn get_name(&self) -> Option<&str> 
  {
    unsafe {
      let c_name = LLVMGetValueName(self.into());
      util::to_null_str(c_name as *mut i8)
    }
  }
  
  /// Sets the name of this value
  pub fn set_name(&self, name: &str) 
  {
    let c_name = CString::new(name).unwrap();
    unsafe {
      LLVMSetValueName(self.into(), c_name.as_ptr())
    }
  }
  
  /// Returns the type of this value
  pub fn get_type(&self) -> &Type 
  {
    unsafe { LLVMTypeOf(self.into()) }.into()
  }
}


pub struct GlobalValue;
native_ref!(&GlobalValue = LLVMValueRef);
deref!(GlobalValue, Value);

impl GlobalValue 
{
	/// Sets the initial value for this global.
	pub fn set_initializer(&self, value: &Value) 
	{
	  unsafe { LLVMSetInitializer(self.into(), value.into()) }
	}
	
	/// Gets the initial value for this global.
	pub fn get_initializer(&self) -> &Value 
	{
	  unsafe { LLVMGetInitializer(self.into()) }.into()
	}
}

impl CastFrom for GlobalValue 
{
  type From = Value;
    
  fn cast<'a>(val: &'a Value) -> Option<&'a GlobalValue>
  {
    let gv = unsafe { LLVMIsAGlobalValue(val.into()) };
    if gv == ptr::null_mut() {
      None
    } else {
      Some(gv.into())
    }
  }
}


/// Comparative operations on values.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Predicate 
{
  Equal,
  NotEqual,
  GreaterThan,
  GreaterThanOrEqual,
  LessThan,
  LessThanOrEqual
}


/// A function argument.
pub struct Arg;
native_ref!(&Arg = LLVMValueRef);

impl Deref for Arg 
{
  type Target = Value;
  
  fn deref(&self) -> &Value 
  {
    unsafe { mem::transmute(self) }
  }
}

impl Arg 
{
  /// Add the attribute given to this argument.
  pub fn add_attribute(&self, attr: Attribute) 
  {
    unsafe { LLVMAddAttribute(self.into(), attr.into()) }
  }
  
  /// Add all the attributes given to this argument.
  pub fn add_attributes(&self, attrs: &[Attribute]) 
  {
    let mut sum = LLVMAttribute::empty();
    for attr in attrs {
      let attr:LLVMAttribute = (*attr).into();
      sum = sum | attr;
    }
    unsafe { LLVMAddAttribute(self.into(), sum.into()) }
  }
  
  /// Returns true if this argument has the attribute given.
  pub fn has_attribute(&self, attr: Attribute) -> bool 
  {
    unsafe {
      let other = LLVMGetAttribute(self.into());
      other.contains(attr.into())
    }
  }
  
  /// Returns true if this argument has all the attributes given.
  pub fn has_attributes(&self, attrs: &[Attribute]) -> bool 
  {
    unsafe {
      let other = LLVMGetAttribute(self.into());
      for &attr in attrs {
        if !other.contains(attr.into()) {
            return false;
        }
      }
      return true;
    }
  }
  
  /// Remove an attribute from this argument.
  pub fn remove_attribute(&self, attr: Attribute) 
  {
    unsafe { 
    	LLVMRemoveAttribute(self.into(), attr.into())
    }
  }
}


/// A function that can be called and contains blocks.
pub struct Function;
native_ref!(&Function = LLVMValueRef);

impl Deref for Function 
{
  type Target = Value;
  
  fn deref(&self) -> &Value 
  {
    unsafe { mem::transmute(self) }
  }
}

impl Index<usize> for Function 
{
  type Output = Arg;
  
  fn index(&self, index: usize) -> &Arg 
  {
    unsafe {
      if index < LLVMCountParams(self.into()) as usize {
        LLVMGetParam(self.into(), index as c_uint).into()
      } else {
        panic!("no such index {} on {:?}", index, self.get_type())
      }
    }
  }
}

impl CastFrom for Function 
{
  type From = Value;
  
  fn cast<'a>(val: &'a Value) -> Option<&'a Function> 
  {
    let ty = val.get_type();
    let mut is_func = ty.is_function();
    
    if let Some(elem) = ty.get_element() {
      is_func = is_func || elem.is_function()
    }
    
    if is_func {
      Some(unsafe { mem::transmute(val) })
    } else {
      None
    }
  }
}

impl Function 
{
  /// Add a basic block with the name given to the function and return it.
  pub fn append<'a>(&'a self, name: &str) -> &'a BasicBlock 
  {
    util::with_cstr(name, |ptr| unsafe {
      LLVMAppendBasicBlockInContext(self.get_context().into(), self.into(), ptr).into()
    })
  }
  
  /// Returns the entry block of this function or `None` if there is none.
  pub fn get_entry(&self) -> Option<&BasicBlock> 
  {
    unsafe { mem::transmute(LLVMGetEntryBasicBlock(self.into())) }
  }
  
  /// Returns the name of this function.
  pub fn get_name(&self) -> &str 
  {
    unsafe {
      let c_name = LLVMGetValueName(self.into());
      util::to_str(c_name as *mut i8)
    }
  }
  
  /// Returns the function signature representing this function's signature.
  pub fn get_signature(&self) -> &FunctionType
  {
    unsafe {
      let ty = LLVMTypeOf(self.into());
      LLVMGetElementType(ty).into()
    }
  }
  
  /// Returns the number of function parameters
  pub fn num_params(&self) -> usize
  {
    unsafe {
      LLVMCountParams(self.into()) as usize
    }
  }
  
  /// Add the attribute given to this function.
  pub fn add_attribute(&self, attr: Attribute) 
  {
    unsafe { LLVMAddFunctionAttr(self.into(), attr.into()) }
  }
  
  /// Add all the attributes given to this function.
  pub fn add_attributes(&self, attrs: &[Attribute]) 
  {
    let mut sum = LLVMAttribute::empty();
    
    for attr in attrs {
        let attr:LLVMAttribute = (*attr).into();
        sum = sum | attr;
    }
    
    unsafe { LLVMAddFunctionAttr(self.into(), sum.into()) }
  }
  
  /// Returns true if the attribute given is set in this function.
  pub fn has_attribute(&self, attr: Attribute) -> bool 
  {
    unsafe {
      let other = LLVMGetFunctionAttr(self.into());
      other.contains(attr.into())
    }
  }
  
  /// Returns true if all the attributes given is set in this function.
  pub fn has_attributes(&self, attrs: &[Attribute]) -> bool 
  {
    unsafe {
      let other = LLVMGetFunctionAttr(self.into());
      
      for &attr in attrs {
          if !other.contains(attr.into()) {
              return false;
          }
      }
      
      return true;
    }
  }
  
  /// Remove the attribute given from this function.
  pub fn remove_attribute(&self, attr: Attribute) 
  {
    unsafe { LLVMRemoveFunctionAttr(self.into(), attr.into()) }
  }
}

impl<'a> IntoIterator for &'a Function 
{
  type Item = &'a Arg;
  type IntoIter = ValueIter<'a, &'a Arg>;
  
  /// Iterate through the functions in the module
  fn into_iter(self) -> ValueIter<'a, &'a Arg> 
  {    
 		ValueIter::new(
 			unsafe { LLVMGetFirstParam(self.into()) },
 			LLVMGetNextParam)  	
  }
}

impl GetContext for Function 
{
  fn get_context(&self) -> &Context 
  {
    self.get_type().get_context()
  }
}


/// A way of indicating to LLVM how you want arguments / functions to be handled.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(C)]
pub enum Attribute 
{
  /// Zero-extended before or after call.
  ZExt =              0b1,
  /// Sign-extended before or after call.
  SExt =              0b10,
  /// Mark the function as not returning.
  NoReturn =          0b100,
  /// Force argument to be passed in register.
  InReg =             0b1000,
  /// Hidden pointer to structure to return.
  StructRet =         0b10000,
  /// Function doesn't unwind stack.
  NoUnwind =          0b100000,
  /// Consider to not alias after call.
  NoAlias =           0b1000000,
  /// Pass structure by value.
  ByVal =             0b10000000,
  /// Nested function static chain.
  Nest =              0b100000000,
  /// Function doesn't access memory.
  ReadNone =          0b1000000000,
  /// Function only reads from memory.
  ReadOnly =          0b10000000000,
  /// Never inline this function.
  NoInline =          0b100000000000,
  /// Always inline this function.
  AlwaysInline =      0b1000000000000,
  /// Optimize this function for size.
  OptimizeForSize =   0b10000000000000,
  /// Stack protection.
  StackProtect =      0b100000000000000,
  /// Stack protection required.
  StackProtectReq =   0b1000000000000000,
  /// Alignment of parameter (5 bits) stored as log2 of alignment with +1 bias 0 means unaligned (different from align(1)).
  Alignment =         0b10000000000000000,
  /// Function creates no aliases of pointer.
  NoCapture =         0b100000000000000000,
  /// Disable redzone.
  NoRedZone =         0b1000000000000000000,
  /// Disable implicit float instructions.
  NoImplicitFloat =   0b10000000000000000000,
  /// Naked function.
  Naked =             0b100000000000000000000,
  /// The source language has marked this function as inline.
  InlineHint =        0b1000000000000000000000,
  /// Alignment of stack for function (3 bits) stored as log2 of alignment with +1 bias 0 means unaligned (different from alignstack=(1)).
  StackAlignment =    0b11100000000000000000000000000,
  /// This function returns twice.
  ReturnsTwice =      0b100000000000000000000000000000,
  /// Function must be in unwind table.
  UWTable =           0b1000000000000000000000000000000,
  /// Function is called early/often, so lazy binding isn't effective.
  NonLazyBind =       0b10000000000000000000000000000000
}

impl From<LLVMAttribute> for Attribute 
{
  fn from(attr: LLVMAttribute) -> Attribute 
  {
    unsafe { mem::transmute(attr) }
  }
}

impl From<Attribute> for LLVMAttribute 
{
  fn from(attr: Attribute) -> LLVMAttribute 
  {
    unsafe { mem::transmute(attr) }
  }
}

impl GetContext for Value 
{
  fn get_context(&self) -> &Context 
  {
    self.get_type().get_context()
  }
}


/// Value Iterator implementation.
///
/// T can be all descendent types of LLVMValueRef.  
#[derive(Copy, Clone)]
pub struct ValueIter<'a, T: From<LLVMValueRef>> {
  cur    : LLVMValueRef,
  step   : unsafe extern "C" fn(LLVMValueRef) -> LLVMValueRef,
  marker1: ::std::marker::PhantomData<&'a ()>,
  marker2: ::std::marker::PhantomData<T>,
}

impl<'a, T: From<LLVMValueRef>> ValueIter<'a, T>
{
	pub fn new(cur: LLVMValueRef, 
		         step: unsafe extern "C" fn(LLVMValueRef) -> LLVMValueRef) -> Self
	{
		ValueIter {
			cur: cur,
			step: step,
			marker1: ::std::marker::PhantomData,
			marker2: ::std::marker::PhantomData
		}
	}
}

impl<'a, T: From<LLVMValueRef>> Iterator for ValueIter<'a, T> 
{
  type Item = T;

  fn next(&mut self) -> Option<T> 
  {
    let old: LLVMValueRef = self.cur;
    
    if !old.is_null() {
      self.cur = unsafe { (self.step)(old) };
      Some(old.into())
    } else {
      None
    }
  }
}