use calc::units::{
    AMPERE, DAY, DEGREE, DEG_FAHRENHEIT, FOOT, INCH, KILOAMPERE, KILOGRAM, MEGAMETER, METER, MOLE,
    NANOCANDELA, NANOSECOND, SECOND,
};
use calc::{eval, popf};

/// Convert a quantity in meters into feet. This is a simple multiplication.
#[test]
fn simple_unit_conversion() {
    let mut ctx = eval::Context::new();
    assert_eq!(ctx.eval("374.4 m ft into"), eval::Status::Ok);

    let f = popf!(ctx.stack).unwrap_or_else(|e| panic!("popf: {e:?}"));
    assert_eq!(f.value, 1228.3464566929133);

    let u = f.unit.unwrap();
    assert_eq!(u.numer(), &[FOOT]);
    assert_eq!(u.denom(), &[]);
}

/// Convert a quantity in microkelvin per nanoradian to degrees Fahrenheit per
/// degree. This is two multiplications.
#[test]
fn derived_unit_conversion() {
    let mut ctx = eval::Context::new();
    assert_eq!(
        ctx.eval("-4.71042 uK nrad / degF deg / into"),
        eval::Status::Ok
    );

    let f = popf!(ctx.stack).unwrap_or_else(|e| panic!("popf: {e:?}"));
    assert_eq!(f.value, -147.9822086732243);

    let u = f.unit.unwrap();
    assert_eq!(u.numer(), &[DEG_FAHRENHEIT]);
    assert_eq!(u.denom(), &[DEGREE]);
}

/// Multiply a quantity in nanocandela degrees per megameter by a quantity in
/// megameter amperes per day mole. The result should be in nanocandela degree
/// amperes per day mole. This involves canceling identical units but no
/// scaling; the magnitude of the result is simply the multiplied magnitudes of
/// the operands.
#[test]
fn simple_unit_simplification_with_multiplication() {
    let mut ctx = eval::Context::new();
    assert_eq!(
        ctx.eval("468.485 ncd deg * Mm / -55.4856 Mm A * day / mol / *"),
        eval::Status::Ok
    );

    let f = popf!(ctx.stack).unwrap_or_else(|e| panic!("popf: {e:?}"));
    assert_eq!(f.value, -25994.171316);

    let u = f.unit.unwrap();
    assert_eq!(u.numer(), &[NANOCANDELA, DEGREE, AMPERE]);
    assert_eq!(u.denom(), &[DAY, MOLE]);
}

/// As above but with division. The result should be in nanocandela degree day
/// moles per megameters squared amperes.
#[test]
fn simple_unit_simplification_with_division() {
    let mut ctx = eval::Context::new();
    assert_eq!(
        ctx.eval("468.485 ncd deg * Mm / -55.4856 Mm A * day / mol / /"),
        eval::Status::Ok
    );

    let f = popf!(ctx.stack).unwrap_or_else(|e| panic!("popf: {e:?}"));
    assert_eq!(f.value, -8.443361881280909);

    let u = f.unit.unwrap();
    assert_eq!(u.numer(), &[NANOCANDELA, DEGREE, DAY, MOLE]);
    assert_eq!(u.denom(), &[MEGAMETER, MEGAMETER, AMPERE]);
}

/// Multiply a quantity in kiloampere nanoseconds per inch mol by a quantity in
/// teraampere kilometers per hour millimole. The result should be in
/// kiloamperes squared per mole squared. This involves canceling like physical
/// quantities and scaling; the magnitude of the result is not simply the
/// multiplied magnitudes of the operands.
#[test]
fn complex_unit_simplification_with_multiplication() {
    let mut ctx = eval::Context::new();
    assert_eq!(
        ctx.eval("-414.485 kA ns * in / mol / -267.36 TA km * hr / mmol / *"),
        eval::Status::Ok
    );

    let f = popf!(ctx.stack).unwrap_or_else(|e| panic!("popf: {e:?}"));
    println!("{f}");
    assert_eq!(f.value, 1211906272.9658796);

    let u = f.unit.unwrap();
    assert_eq!(u.numer(), &[KILOAMPERE, KILOAMPERE]);
    assert_eq!(u.denom(), &[MOLE, MOLE]);
}

/// As above but with division. Result should be in nanoseconds squared per
/// inches squared.
#[test]
fn complex_unit_simplification_with_division() {
    let mut ctx = eval::Context::new();
    assert_eq!(
        ctx.eval("-414.485 kA ns * in / mol / -267.36 TA km * hr / mmol / /"),
        eval::Status::Ok
    );

    let f = popf!(ctx.stack).unwrap_or_else(|e| panic!("popf: {e:?}"));
    println!("{f}");
    assert_eq!(f.value, 0.00014175833482944343);

    let u = f.unit.unwrap();
    assert_eq!(u.numer(), &[NANOSECOND, NANOSECOND]);
    assert_eq!(u.denom(), &[INCH, INCH]);
}

/// Tests that derived units of different magnitudes cancel correctly. This
/// exercises factors on `Unit`.
#[test]
fn unit_factors_multiplication() {
    let mut ctx = eval::Context::new();
    assert_eq!(ctx.eval("-53.3719 MA 99.5531 fohm *"), eval::Status::Ok);

    let f = popf!(ctx.stack).unwrap_or_else(|e| panic!("popf: {e:?}"));
    println!("{f}");
    assert_eq!(f.value, -5.31333809789e-6);

    let u = f.unit.unwrap();
    assert_eq!(u.numer(), &[KILOGRAM, METER, METER]);
    assert_eq!(u.denom(), &[SECOND, SECOND, SECOND, AMPERE]);
}

/// As above but with division.
#[test]
fn unit_factors_division() {
    let mut ctx = eval::Context::new();
    assert_eq!(ctx.eval("-53.3719 MV 99.5531 fohm /"), eval::Status::Ok);

    let f = popf!(ctx.stack).unwrap_or_else(|e| panic!("popf: {e:?}"));
    println!("{f}");
    assert_eq!(f.value, -5.361148974768238e20);

    let u = f.unit.unwrap();
    assert_eq!(u.numer(), &[AMPERE]);
    assert_eq!(u.denom(), &[]);
}

#[test]
fn psi_to_pascals() {
    let mut ctx = eval::Context::new();
    assert_eq!(ctx.eval("344.6790593239441 psi Pa into"), eval::Status::Ok);

    let f = popf!(ctx.stack).unwrap_or_else(|e| panic!("popf: {e:?}"));
    println!("{f}");
    assert_eq!(f.value, 2376478.4580761734);

    let u = f.unit.unwrap();
    assert_eq!(u.numer(), &[KILOGRAM]);
    assert_eq!(u.denom(), &[SECOND, SECOND, METER]);
}
