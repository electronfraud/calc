"""
Generates code to define selected SI-prefixed units.
"""

UNITS = (
    ('SECOND', 's', 'Time', 'base'),
    ('METER', 'm', 'Length', 'base'),
    ('GRAM', 'g', 'Mass', 'base'),
    ('AMPERE', 'A', 'Current', 'base'),
    ('KELVIN', 'K', 'Temperature', 'base'),
    ('MOLE', 'mol', 'AmountOfSubstance', 'base'),
    ('CANDELA', 'cd', 'LuminousIntensity', 'base'),
    ('RADIAN', 'rad', 'Angle', 'base'),
    ('JOULE', 'J', None, 'unit'),
    ('NEWTON', 'N', None, 'unit'),
    ('WATT', 'W', None, 'unit'),
    ('PASCAL', 'Pa', None, 'unit'),
    ('VOLT', 'V', None, 'unit'),
    ('OHM', 'ohm', None, 'unit'),
    ('FARAD', 'F', None, 'unit'),
    ('TESLA', 'T', None, 'unit'))


PREFIXES = (
    ('PETA', 'P', '1e15'),
    ('TERA', 'T', '1e12'),
    ('GIGA', 'G', '1e9'),
    ('MEGA', 'M', '1e6'),
    ('KILO', 'k', '1e3'),
    ('MILLI', 'm', '1e-3'),
    ('MICRO', 'u', '1e-6'),
    ('NANO', 'n', '1e-9'),
    ('PICO', 'p', '1e-12'),
    ('FEMTO', 'f', '1e-15'))

# This section goes in src/units/mod.rs
print('// SI prefixes')

for name, symbol, physq, kind in UNITS:
    for namepfx, sympfx, factor in PREFIXES:
        if namepfx == 'KILO' and name == 'GRAM':
            continue
        if namepfx == 'MILLI' and name == 'METER':
            print(f'pub static CENTI{name}: Base = Base::new("c{symbol}", {physq}, 1e-2);')
        if kind == 'base':
            print(f'pub static {namepfx}{name}: Base = Base::new("{sympfx}{symbol}", {physq}, {factor});')
        else:
            print(f'pub static {namepfx}{name}: Lazy<Unit> = Lazy::new(|| {name}.with_factor({factor}).with_symbol("{sympfx}{symbol}"));');
    print()

# This section goes in src/builtins.rs
print('        // SI prefixes')
for name, _, _, kind in UNITS:
    for namepfx, _, _ in PREFIXES:
        if namepfx == 'KILO' and name == 'GRAM':
            continue
        macro = 'base!(' if kind == 'base' else 'unit!(&*'
        if namepfx == 'MILLI' and name == 'METER':
            print(f'        {macro}units::CENTI{name}),')
        print(f'        {macro}units::{namepfx}{name}),')
