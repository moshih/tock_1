#![feature(asm,concat_idents,const_fn)]
#![no_std]

extern crate kernel;
use kernel::common::volatile_cell::VolatileCell;

#[repr(C, packed)]
pub struct CountRegisters {
    // From page 1005 of SAM4L manual
    
    DWT_CONTROL  : VolatileCell<u32>, // Mode        (0x04)
    DWT_CYCCNT   : VolatileCell<u32>, // Control               (0x00)

}



// Page 59 of SAM4L data sheet
const BASE_ADDRESS: *mut CountRegisters = 0xE0001000 as *mut CountRegisters;

pub struct cycle_counter_inst { //aes_dev_inst
    registers: *mut CountRegisters, //Aesa *hw_dev;

}


pub static mut CYCLE_counter_inst: cycle_counter_inst = cycle_counter_inst::new(BASE_ADDRESS);



impl cycle_counter_inst {
    const fn new(base_address: *mut CountRegisters) -> cycle_counter_inst {
        cycle_counter_inst {
            registers: base_address,
        }
    }
   
    
    pub fn reset (&self){
 
        unsafe { (*self.registers).DWT_CONTROL.set(0) };
        unsafe { (*self.registers).DWT_CYCCNT.set(0) };
        
        unsafe{
            let SCB_DEMCR = 0xE000EDFC as *mut i32;
            *SCB_DEMCR=*SCB_DEMCR |0x01000000;
        
        }

    
    }
    
    pub fn start_timer (&self){
 
        unsafe { (*self.registers).DWT_CONTROL.set( (*self.registers).DWT_CONTROL.get()|1) };


    
    }
    
    pub fn stop_timer (&self){
 
        unsafe { (*self.registers).DWT_CONTROL.set( (*self.registers).DWT_CONTROL.get()& 0xFFFFFFFE) };


    
    }
    
    

    pub fn get_cycles(&self) -> u32{
    
        //for x in 0..4 {
            unsafe { return ((*self.registers).DWT_CYCCNT.get()) };
        //}   databufptr
    }
}

pub fn test(){

}
