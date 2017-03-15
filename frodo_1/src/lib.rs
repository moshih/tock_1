#![feature(asm,concat_idents,const_fn)]
#![no_std]

//extern crate kernel;



//extern fn lwe_sample_n_inverse_12(in:&mut[u16;64]);
mod d_3;

pub fn rand_noise(){
    let mut ina:[u16;64]=[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1];

    lwe_sample_n_inverse_12v(&mut ina);
}
