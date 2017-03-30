#![feature(asm,concat_idents,const_fn)]
#![no_std]
#![feature(type_ascription)]

//extern crate kernel;

extern crate sam4l;
use sam4l::aesa;

//extern fn lwe_sample_n_inverse_12(in:&mut[u32;64]);
mod d_3;

use d_3::lwe_sample_n_inverse_12_slice;


const B:u32=4;
const B_bar:u32=11;
const two_pow_B:u32=16;
const two_pow_B_bar:u32=2048;
const q:u32=32768;

const n:u32=752;
const n_bar:u32=8;
const m_bar:u32=8;

pub fn rounding (v:u32) -> u32{
    let calculation:f64=  (v as f64)/(two_pow_B_bar as f64)+0.5;
    

    let output:u32 = (calculation as u32)%two_pow_B;
    return output;
}

pub fn equal(inputa:u32,inputb:u32) -> u32{
    let mut output:u32=0;
    let xor:u32=inputa^inputb;
    for i in 0..32 {
        output=output| (xor<<i>>31);
    }
    return 1-output;

}

pub fn cross (v:u32) -> u32{
    let calculation:f64= ( 2.0*v as f64)/(two_pow_B_bar as f64);
    

    let output:u32 = (calculation as u32)%2;
    return output;
}


pub fn reconciliation (w:u32, b:u32) -> u32{
    let gate_a:u32=((equal(cross(w),b)) as u32);
    let gate_b:u32= ((equal(w&512,0)) as u32);
    let pre_result:u32=((w>>10)<<10);
    let result_a=(pre_result+q-1)%q;
    let result_b = (two_pow_B_bar/2 +pre_result);
    let result = gate_a*w+ (1-gate_a)*(gate_b*result_a +(1-gate_b)*result_b   );
    return rounding(result);
}

pub fn gen_a_slice(key128:&[u32; 4], A:&mut[u32; (n) as usize ]){

    for i in 0..n/4-1{
        unsafe {
            let aesa_temp = &sam4l::aesa::AES_dev_inst;
            aesa_temp.aes_get_config_defaults();
            aesa_temp.aes_set_enable();
            aesa_temp.aes_set_new_message();
            aesa_temp.aes_write_key(&key128);
            
            let  plain_text:[u32; 4] = [ 4*i, 4*i+1,4*i+2,4*i+3];
            aesa_temp.aes_write_input_data(plain_text[0]);
            aesa_temp.aes_write_input_data(plain_text[1]);
            aesa_temp.aes_write_input_data(plain_text[2]);
            aesa_temp.aes_write_input_data(plain_text[3]);

           
            
            while aesa_temp.aes_done()!= 1 {
            }
            A[ (4*i) as usize ]  = aesa_temp.aes_read_output_data();
            A[ (4*i+1) as usize ]  = aesa_temp.aes_read_output_data();
            A[ (4*i+2) as usize ]  = aesa_temp.aes_read_output_data();
            A[ (4*i+3) as usize ]  = aesa_temp.aes_read_output_data();
        
        }
    

    
    }
        
}


pub fn rand_noise( index:u16) -> u32{
    let mut ina:[u32;(n) as usize]=[0; (n) as usize];
    lwe_sample_n_inverse_12_slice(&mut ina);
    return ina[index as usize];
}




