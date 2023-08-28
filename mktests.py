"""
Generates fuzz tests for unit conversions, with values produced by `units`.
"""

import random
import re
import subprocess

def units(x, from_unit, to_unit):
    """
    Run `units` to convert `x` from `from_unit` to `to_unit` and return the
    result as a string. Raises a ValueError if `units` fails or indicates a
    reciprocal conversion.
    """
    ps = subprocess.run(['units',
                         '-1',
                         '-o', '%.16g',
                         f'{x} {from_unit}',
                         to_unit],
                        capture_output=True)
    if ps.returncode != 0 or b'reciprocal conversion' in ps.stdout:
        raise ValueError
    return str(ps.stdout, encoding='ascii').split(' ')[1].rstrip()

# Load the list of all available units by reading src/units/mod.rs.
BASE_RE = re.compile(r'Base::new\("([^"]+)')
UNIT_RE = re.compile(r'with_symbol\("([^"]+)')

mod_rs_without_newlines = ''
with open('src/units/mod.rs', 'rt') as f:
    for line in f.readlines():
        line = (line.split('//', 2)[0]
                    .split('#![', 2)[0]
                    .split('#[', 2)[0]
                    .strip())
        mod_rs_without_newlines += line + ' '

all_units = []
for stmt in mod_rs_without_newlines.split(';'):
    m = BASE_RE.search(stmt) or UNIT_RE.search(stmt)
    if m:
        all_units.append(m[1])

# Generate a random unit conversion test.
while True:
    u1 = (
        random.choice(all_units),
        random.choice(all_units),
        random.choice(all_units))
    u2 = (
        random.choice(all_units),
        random.choice(all_units))
    op1 = random.choice(('*', '/'))
    op2 = random.choice(('*', '/'))
    op3 = random.choice(('*', '/'))

    x = f'{random.random()*1000 - 500:.16f}'

    try:
        y = units(x, f'{u1[0]}{op1}{u1[1]}{op2}{u1[2]}', f'{u2[0]}{op3}{u2[1]}')
    except ValueError:
        continue

    print(f'''#[test]
#[allow(non_snake_case)]
fn test_{u1[0]}_{u1[1]}_{u1[2]}_{u2[0]}_{u2[1]}() {{
    dotest!("{x} {u1[0]} {u1[1]} {op1} {u1[2]} {op2} {u2[0]} {u2[1]} {op3} into", {y});
}}''')

    break
