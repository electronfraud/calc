# calc

[![Rust](https://github.com/electronfraud/calc/actions/workflows/rust.yml/badge.svg)](https://github.com/electronfraud/calc/actions/workflows/rust.yml)

A terminal-based, units-aware, RPN calculator.

![image](https://github.com/electronfraud/calc/assets/126712383/b6a02d8f-bdd5-4df5-90b3-91a5e7002430)

* [How to Use](#how-to-use)
  * [The Stack](#the-stack)
  * [Units](#units)
    * [Temperature](#temperature)
  * [Readline](#readline)
* [Reference](#reference)
  * [Constants](#constants)
  * [Commands](#commands)
    * [Interface](#interface)
    * [Arithmetic](#arithmetic)
    * [Trigonometry](#trigonometry)
    * [Unit Conversion](#unit-conversion)
    * [Stack Manipulation](#stack-manipulation)
  * [Units](#units-1)
* [About](#about)
  * [License](#license)
  * [Future Work](#future-work)

## How to Use

Enter operands followed by the operator. The result is displayed in
the prompt.

```
() 1 2 +
(3)
```

Type `exit`, `q`, or EOF (ctrl-D) to exit.

### The Stack

`calc` performs all operations on a stack. When you enter a number,
the number is pushed onto the top of the stack. When you enter an
operator, it pops operands off the top of the stack, performs the
operation, and pushes the result onto the stack. The contents of
the stack are displayed in the prompt; the rightmost value is the
top of the stack.

Here is the example from above, performed one entry at a time.
First, 1 is pushed onto the stack:

```
() 1
(1)
```

Then 2 is pushed:

```
() 1
(1) 2
(1 2)
```

Then `+` is entered, which pops 1 and 2 and pushes the answer, 3:

```
() 1
(1) 2
(1 2) +
(3)
```

### Units

`calc` can perform arithmetic on quantities of differing units. When
adding and subtracting, the units can differ but must measure the same
physical phenomenon (length, time, temperature, etc.). When multiplying
and dividing, `calc` computes the resulting units.

Numbers on the stack are assigned units by entering the name of the
unit. Numbers with units are displayed in square brackets along with
their unit. For example, to push a value of two inches:

```
() 2 in
([2 in])
```

You could then add or subtract quantities of any other length unit. The
result will have the units of the bottom (first) operand. For example,
the following produces a value of 1.5 inches:

```
() 2 in
([2 in]) 1.27 cm
([2 in] [1.27 cm]) -
([1.5 in])
```

To convert a quantity into different units, use the `into` command:

```
() 2 in
([2 in]) cm into
([5.08 cm])
```

You can, of course, enter everything on one line:

```
() 2 in cm into
([5.08 cm])
```

Multiplication and division compute new units. For example, if a runner
completes a 100-meter dash in 9.58 seconds, dividing these quantities
produces the average speed in meters per second:

```
() 100 m 9.58 s /
([10.438413 m⋅s⁻¹])
```

You could then convert that into miles per hour:

```
() 100 m 9.58 s /
([10.438413 m⋅s⁻¹]) mi hr /
([10.438413 m⋅s⁻¹] mi⋅hr⁻¹) into
([23.350066 mi⋅hr⁻¹])
```

#### Temperature

Temperatures pose some difficulties when it comes to unit conversion because
the conversion depends on whether you mean "the temperature (outside, of
this person, of the sun, etc.) is X degrees" or "a difference of X degrees."
For the first kind of temperature, use `tempC` or `tempF` units; for
differences in temperature, use `degC` or `degF` units. `K` (Kelvin) and
`R` (Rankine) can be used for either because they are based on absolute
zero.

Both types of temperature unit can be converted to and from `K` and `R`,
but `temp` units can't be converted to `deg` units, and vice versa. Also,
`deg` units can be mixed with other kinds of units (e.g. `degC` per `s`)
but `temp` units can't (`tempC` per `s` is nonsensical).

For example, if the temperature outside somewhere in the United States is
78 degrees Fahrenheit and you want to know what that is in Celsius, you
would do:

```
() 78 tempF tempC into
([25.555556 tempC])
```

But if a red-hot nickel ball cooled by 78 degrees Fahrenheit and you
wanted to know how much its temperature had changed in degrees Celsius,
you would do:

```
() 78 degF degC into
([43.333333 degC])
```

### Radices

In addition to base-10 real numbers, you can enter integers in hexadecimal,
octal, and binary. You can also display integers in these bases and use
bitwise operators on them. For example:

```
() 0xeb9f
(0xeb9f) 0b10001101
(0xeb9f 0b10001101) &
(0x8d)
```

To change an integer's display format, enter `hex`, `dec`, `oct`, or `bin`:

```
(0x8d) oct
(0215) bin
(0b10001101) dec
(141)
```

The following formats are accepted:

| Radix | Prefixes        | Separator | Example               |
|-------|-----------------|-----------|-----------------------|
| 16    | `0x`, `0X`, `$` | `_`       | `0x12345678_abcdef01` |
| 10    | None            | `,`       | `123,456,789,012`     |
| 8     | `0`, `0o`, `0O` | `_`       | `0123_456_701`        |
| 2     | `0b`, `0B`      | `_`       | `0b10101010_10101010` |

### Readline

`calc` has all the usual readline affordances: tab completion, history,
and line editing. To autocomplete a command or unit, or to see the
available completions, type at least one letter and press tab. To reenter
a previous command, use the up and down arrows to scroll through history.
History is saved between sessions. To edit a line while entering it, use
the left and right arrows.

## Reference

### Constants

There are a few built-in constants. To push one, enter its name.

| Name   | Description                                       | Value                 |
|--------|---------------------------------------------------|-----------------------|
| `c`    | Speed of light in a vacuum                        | 299,792,458 m⋅s⁻¹     |
| `e`    | Euler's number                                    | 2.718281828459045     |
| `h`    | Planck constant                                   | 6.62607015e-34 J⋅Hz⁻¹ |
| `hbar` | Reduced Planck constant                           | 1.054571817e-34 J⋅s   |
| `pi`   | Ratio of a circle's circumference to its diameter | 3.141592653589793     |

### Commands

This is a list of all available commands.

#### Interface

| Name    | Description       |
|---------|-------------------|
| `exit`  | Exit the program. |
| `q`     | Exit the program. |

#### Arithmetic

| Name    | Description     |
|---------|-----------------|
| `+`     | Addition.       |
| `-`     | Subtraction.    |
| `*`     | Multiplication. |
| `/`     | Division.       |

#### Trigonometry

| Name    | Description                                       |
|---------|---------------------------------------------------|
| `sin`   | Sine. Accepts any angle unit.                     |
| `cos`   | Cosine. Accepts any angle unit.                   |
| `tan`   | Tangent. Accepts any angle unit.                  |
| `asin`  | Arc sine. Result has units of `rad` (radians).    |
| `acos`  | Arc cosine. Result has units of `rad` (radians).  |
| `atan`  | Arc tangent. Result has units of `rad` (radians). |

#### Unit Conversion

| Name    | Description                            |
|---------|----------------------------------------|
| `drop`  | Remove the units from a number.        |
| `into`  | Convert a number into different units. |

#### Bitwise and Binary Integer Operations

| Name  | Description                        |
|-------|------------------------------------|
| `&`   | Bitwise AND.                       |
| `\|`   | Bitwise OR.                       |
| `^`   | Bitwise XOR.                       |
| `~`   | Bitwise complement.                |
| `hex` | Display an integer in hexadecimal. |
| `dec` | Display an integer in decimal.     |
| `oct` | Display an integer in octal.       |
| `bin` | Display an integer in binary.      |

#### Stack Manipulation

| Name    | Description                                 |
|---------|---------------------------------------------|
| `clear` | Empty the stack.                            |
| `dup`   | Duplicate the item on top of the stack.     |
| `keep`  | Empty the stack except for the top N items. |
| `pop`   | Pop an item off the stack.                  |
| `swap`  | Swap the top two items on the stack.        |

### Units

The following units are supported.

| Symbol  | Description                                                                 |
|---------| ----------------------------------------------------------------------------|
| `degC`  | Degrees Celsius. This unit is an interval. For temperature, use `tempC`.    |
| `degF`  | Degrees Fahrenheit. This unit is an interval. For temperature, use `tempF`. |
| `K`     | Kelvin. The SI base unit of temperature.                                    |
| `kg`    | Kilogram. The SI base unit of mass.                                         |
| `m`     | Meter. The SI base unit of length.                                          |
| `R`     | Rankine. Like Kelvin, but for Fahrenheit.                                   |
| `s`     | Second. The SI base unit of time.                                           |
| `tempC` | Temperature in degrees Celsius. For intervals, use `degC`.                  |
| `tempF` | Temperature in degrees Fahrenheit. For intervals, use `degF`.               |

## About

`calc`'s functionality is influenced by Adrian Mariano's `units` and
the Forth programming language.

### License

Copyright 2023 electronfraud (she/her)

This program is free software: you can redistribute it and/or modify it
under the terms of the GNU General Public License as published by the
Free Software Foundation, either version 3 of the License, or (at your
option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program. If not, see <https://www.gnu.org/licenses/>.

### Future Work

- Basics
  - Exponent operator
  - Additional units
  - Complex numbers
- Conveniences
  - Adjustable output precision
  - Inline help
  - Configuration
- Enablers
  - Variables
  - Word (function) definition
  - Preamble/libraries
- Stretch
  - Matrix operations
  - Symbolic computation
  - Graphing
