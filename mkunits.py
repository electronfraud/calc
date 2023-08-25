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
    ('RADIAN', 'rad', 'Angle', 'base'))

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
            # TODO
            pass
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
