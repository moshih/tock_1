

pub fn lwe_sample_n_inverse_12(input:&mut[u32;64]){
    let CDF_LENGTH_D3:u8 = 6;
    let CDF_D3:[u32;6] = [602, 1521, 1927, 2031, 2046, 2047];


    for i in 0..64 {
        let mut sample:u8=0;
        let rnd:u8 = (input[i] as u8) >>1;
        let sign:u8 = (input[i] as u8)&1;
        
        for j in 0..CDF_LENGTH_D3-1 {
            sample+= ((CDF_D3[0] as u8)-rnd)>>7;
        
        }
        //flips sample if sign==1, does nothign if sample==0
        input[i] = ((((255+(sign as u32))^255) as u8) ^sample) as u32;


    }
}

