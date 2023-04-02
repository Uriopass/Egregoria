const EXP_C: f32 = 87.0;

fn calc_exp(z: f32) -> f32 {
    return exp(EXP_C * z - EXP_C);
}