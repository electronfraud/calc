use approx::assert_relative_eq;

use calc::eval;
use calc::popf;

macro_rules! dotest {
    ($code:expr, $res:expr) => {
        let mut ctx = eval::Context::new();
        let status = ctx.eval($code);
        assert_eq!(status, eval::Status::Ok);
        let mut tx = ctx.stack.begin();
        assert_relative_eq!(
            popf!(tx).unwrap().value,
            $res,
            epsilon = f64::EPSILON * 100.0
        );
    };
}

// TODO
/*
#[test]
#[allow(non_snake_case)]
fn test_GPa_fV_T_TW_mi() {
    dotest!(
        "464.7714026087800221 GPa fV * T / TW mi / into",
        0.000000000000748
    );
}
*/

#[test]
#[allow(non_snake_case)]
fn test_GJ_pPa_MN_mm_Pa() {
    dotest!(
        "288.7384336113130985 GJ pPa / MN / mm Pa / into",
        288738433611313098500.0
    );
}

#[test]
#[allow(non_snake_case)]
fn test_day_Gohm_ft_yd_uF() {
    dotest!(
        "466.7865650752054307 day Gohm / ft * yd uF * into",
        13443.453074165916405
    );
}

#[test]
#[allow(non_snake_case)]
fn test_uA_J_PJ_TV_kohm() {
    dotest!("472.5191758644920128 uA J * PJ / TV kohm / into", 0.0);
}

#[test]
#[allow(non_snake_case)]
fn test_mN_MJ_PN_cm_nN() {
    dotest!(
        "317.0780678280855227 mN MJ * PN / cm nN * into",
        31.7078067828085466
    );
}

#[test]
#[allow(non_snake_case)]
fn test_V_TJ_s_kV_W() {
    dotest!(
        "105.9802161443672048 V TJ / s * kV W / into",
        0.0000000000001060
    );
}

// TODO: This has an intermediate product, 307.568688 MA/A, that is technically
// correct but not properly simplified.
#[test]
#[allow(non_snake_case)]
fn test_nohm_fV_MA_kPa_MPa() {
    dotest!(
        "307.5686876719602196 nohm fV / MA * kPa MPa / *",
        307568687671.960256
    );
}

#[test]
#[allow(non_snake_case)]
fn test_fW_kA_kPa_Pa_PV() {
    dotest!(
        "39.2703942533396457 fW kA / kPa * Pa PV * into",
        3.92703942533396457e-29
    );
}

#[test]
#[allow(non_snake_case)]
fn test_ns_Trad_mohm_uF_urad() {
    dotest!(
        "279.8736860245013531 ns Trad / mohm / uF urad / into",
        0.0000000000000003
    );
}
