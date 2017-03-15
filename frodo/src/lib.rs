#![feature(asm,concat_idents,const_fn)]
#![no_std]
#![feature(type_ascription)]

//extern crate kernel;

extern crate sam4l;
use sam4l::aesa;

//extern fn lwe_sample_n_inverse_12(in:&mut[u16;64]);
mod d_3;
use d_3::lwe_sample_n_inverse_12;

const B:u16=4;
const B_bar:u16=11;
const two_pow_B:u16=16;
const two_pow_B_bar:u16=2048;
const q:u16=32768;

pub fn rounding (v:u16) -> u16{
    let calculation:f64=  (v as f64)/(two_pow_B_bar as f64)+0.5;
    

    let output:u16 = (calculation as u16)%two_pow_B;
    return output;
}


pub fn cross (v:u16) -> u16{
    let calculation:f64= ( 2.0*v as f64)/(two_pow_B_bar as f64);
    

    let output:u16 = (calculation as u16)%2;
    return output;
}


pub fn reconciliation (w:u16, b:u16) -> u16{
    let gate_a:u16=((cross(w)==b) as u16);
    let gate_b:u16= ((w&512==0) as u16);
    let pre_result:u16=((w>>10)<<10);
    let result_a=(pre_result+q-1)%q;
    let result_b = (two_pow_B_bar/2 +pre_result);
    let result = gate_a*w+ (1-gate_a)*(gate_b*result_a +(1-gate_b)*result_b   );
    return rounding(result);
}

pub fn rand_noise(){
    let mut ina:[u16;64]=[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1];

    lwe_sample_n_inverse_12(&mut ina);
}


