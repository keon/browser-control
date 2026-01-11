# awesome-bits [![Awesome](https://cdn.rawgit.com/sindresorhus/awesome/d7305f38d29fed78fa85652e3a63e154dd8e8829/media/badge.svg)](https://github.com/sindresorhus/awesome)

> A curated list of awesome bitwise operations and tricks
>
> Maintainer - [Keon Kim](https://github.com/keonkim)
> Please feel free to [pull requests](https://github.com/keonkim/awesome-bits/pulls)




## Integers
**Set n<sup>th</sup> bit**
```
x | (1<<n)
```
**Unset n<sup>th</sup> bit**
 ```
 x & ~(1<<n)
 ```
**Toggle n<sup>th</sup> bit**
```
x ^ (1<<n)
```
**Round up to the next power of two**
```
unsigned int v; //only works if v is 32 bit
v--;
v |= v >> 1;
v |= v >> 2;
v |= v >> 4;
v |= v >> 8;
v |= v >> 16;
v++;
```
**Round down / floor a number**
```
n >> 0

5.7812 >> 0 // 5

```

**Check if even**
```
(n & 1) == 0
```

**Check if odd**
```
(n & 1) != 0
```
**Get the maximum integer**
```
int maxInt = ~(1 << 31);
int maxInt = (1 << 31) - 1;
int maxInt = (1 << -1) - 1;
int maxInt = -1u >> 1;
```
**Get the minimum integer**
```
int minInt = 1 << 31;
int minInt = 1 << -1;
```
**Get the maximum long**
```
long maxLong = ((long)1 << 127) - 1;
```
**Multiply by 2**
```
n << 1; // n*2
```
**Divide by 2**
```
n >> 1; // n/2
```
**Multiply by the m<sup>th</sup> power of 2**
```
n << m;
```
**Divide by the m<sup>th</sup> power of 2**
```
n >> m;
```
**Check Equality**

<sub>*This is 35% faster in Javascript*</sub>
```
(a^b) == 0; // a == b
!(a^b) // use in an if
```
**Check if a number is odd**
```
(n & 1) == 1;
```
**Exchange (swap) two values**
```
//version 1
a ^= b;
b ^= a;
a ^= b;

//version 2
a = a ^ b ^ (b = a)
```
**Get the absolute value**
```
//version 1
x < 0 ? -x : x;

//version 2
(x ^ (x >> 31)) - (x >> 31);
```
**Get the max of two values**
```
b & ((a-b) >> 31) | a & (~(a-b) >> 31);
```
**Get the min of two values**
```
a & ((a-b) >> 31) | b & (~(a-b) >> 31);
```
**Check whether both numbers have the same sign**
```
(x ^ y) >= 0;
```
**Flip the sign**
```
i = ~i + 1; // or
i = (i ^ -1) + 1; // i = -i
```
**Calculate 2<sup>n</sup>**
```
1 << n;
```
**Whether a number is power of 2**
```
n > 0 && (n & (n - 1)) == 0;
```
**Modulo 2<sup>n</sup> against m**
```
m & ((1 << n) - 1);
```
**Get the average**
```
(x + y) >> 1;
((x ^ y) >> 1) + (x & y);
```
**Get the m<sup>th</sup> bit of n (from low to high)**
```
(n >> (m-1)) & 1;
```
**Set the m<sup>th</sup> bit of n to 0 (from low to high)**
```
n & ~(1 << (m-1));
```
**Check if n<sup>th</sup> bit is set**
```
if (x & (1<<n)) {
  n-th bit is set
} else {
  n-th bit is not set
}
```
**Isolate (extract) the right-most 1 bit**
```
x & (-x)
```
**Isolate (extract) the right-most 0 bit**
```
~x & (x+1)
```

**Set the right-most 0 bit to 1**
```
x | (x+1)
```

**Set the right-most 1 bit to 0**
```
x & (x-1)
```

**n + 1**
```
-~n
```
**n - 1**
```
~-n
```
**Get the negative value of a number**
```
~n + 1;
(n ^ -1) + 1;
```
**`if (x == a) x = b; if (x == b) x = a;`**
```
x = a ^ b ^ x;
```
**Swap Adjacent bits**
```
((n & 10101010) >> 1) | ((n & 01010101) << 1)
```
**Different rightmost bit of numbers m & n**
```
(n^m)&-(n^m) // returns 2^x where x is the position of the different bit (0 based)
```
**Common rightmost bit of numbers m & n**
```
~(n^m)&(n^m)+1 // returns 2^x where x is the position of the common bit (0 based)
```
## Floats

These are techniques inspired by the [fast inverse square root method.](https://en.wikipedia.org/wiki/Fast_inverse_square_root) Most of these
are original.

**Turn a float into a bit-array (unsigned uint32_t)**
```c
#include <stdint.h>
typedef union {float flt; uint32_t bits} lens_t;
uint32_t f2i(float x) {
  return ((lens_t) {.flt = x}).bits;
}
```
<sub>*Caveat: Type pruning via unions is undefined in C++; use `std::memcpy` instead.*</sub>

**Turn a bit-array back into a float**
```c
float i2f(uint32_t x) {
  return ((lens_t) {.bits = x}).flt;
}
```

**Approximate the bit-array of a *positive* float using `frexp`**

*`frexp` gives the 2<sup>n</sup> decomposition of a number, so that `man, exp = frexp(x)` means that man * 2<sup>exp</sup> = x and 0.5 <= man < 1.*
```c
man, exp = frexp(x);
return (uint32_t)((2 * man + exp + 125) * 0x800000);
```
<sub>*Caveat: This will have at most 2<sup>-16</sup> relative error, since man + 125 clobbers the last 8 bits, saving the first 16 bits of your mantissa.*</sub>

**Fast Inverse Square Root**
```c
return i2f(0x5f3759df - f2i(x) / 2);
```
<sub>*Caveat: We're using the `i2f` and the `f2i` functions from above instead.*</sub>

See [this Wikipedia article](https://en.wikipedia.org/wiki/Fast_inverse_square_root#A_worked_example) for reference.

**Fast n<sup>th</sup> Root of positive numbers via Infinite Series**
```c
float root(float x, int n) {
#DEFINE MAN_MASK 0x7fffff
#DEFINE EXP_MASK 0x7f800000
#DEFINE EXP_BIAS 0x3f800000
  uint32_t bits = f2i(x);
  uint32_t man = bits & MAN_MASK;
  uint32_t exp = (bits & EXP_MASK) - EXP_BIAS;
  return i2f((man + man / n) | ((EXP_BIAS + exp / n) & EXP_MASK));
}
```

See [this blog post](http://www.phailed.me/2012/08/somewhat-fast-square-root/) regarding the derivation.

**Fast Arbitrary Power**
```c
return i2f((1 - exp) * (0x3f800000 - 0x5c416) + f2i(x) * exp)
```

<sub>*Caveat: The `0x5c416` bias is given to center the method. If you plug in exp = -0.5, this gives the `0x5f3759df` magic constant of the fast inverse root method.*</sub>

See [these set of slides](http://www.bullshitmath.lol/FastRoot.slides.html) for a derivation of this method.

**Fast Geometric Mean**

The geometric mean of a set of `n` numbers is the n<sup>th</sup> root of their
product.

```c
#include <stddef.h>
float geometric_mean(float* list, size_t length) {
  // Effectively, find the average of map(f2i, list)
  uint32_t accumulator = 0;
  for (size_t i = 0; i < length; i++) {
    accumulator += f2i(list[i]);
  }
  return i2f(accumulator / n);
}
```
See [here](https://github.com/leegao/float-hacks#geometric-mean-1) for its derivation.

**Fast Natural Logarithm**

```c
#DEFINE EPSILON 1.1920928955078125e-07
#DEFINE LOG2 0.6931471805599453
return (f2i(x) - (0x3f800000 - 0x66774)) * EPSILON * LOG2
```

<sub>*Caveat: The bias term of `0x66774` is meant to center the method. We multiply by `ln(2)` at the end because the rest of the method computes the `log2(x)` function.*</sub>

See [here](https://github.com/leegao/float-hacks#log-1) for its derivation.

**Fast Natural Exp**

```c
return i2f(0x3f800000 + (uint32_t)(x * (0x800000 + 0x38aa22)))
```

<sub>*Caveat: The bias term of `0x38aa22` here corresponds to a multiplicative scaling of the base. In particular, it
corresponds to `z` such that 2<sup>z</sup> = e*</sub>

See [here](https://github.com/leegao/float-hacks#exp-1) for its derivation.

## Strings

**Convert letter to lowercase:**
```
OR by space => (x | ' ')
Result is always lowercase even if letter is already lowercase
eg. ('a' | ' ') => 'a' ; ('A' | ' ') => 'a'
```

**Convert letter to uppercase:**
```
AND by underline => (x & '_')
Result is always uppercase even if letter is already uppercase
eg. ('a' & '_') => 'A' ; ('A' & '_') => 'A'
```
**Invert letter's case:**
```
XOR by space => (x ^ ' ')
eg. ('a' ^ ' ') => 'A' ; ('A' ^ ' ') => 'a'
```
**Letter's position in alphabet:**
```
AND by chr(31)/binary('11111')/(hex('1F') => (x & "\x1F")
Result is in 1..26 range, letter case is not important
eg. ('a' & "\x1F") => 1 ; ('B' & "\x1F") => 2
```
**Get letter's position in alphabet (for Uppercase letters only):**
```
AND by ? => (x & '?') or XOR by @ => (x ^ '@')
eg. ('C' & '?') => 3 ; ('Z' ^ '@') => 26
```
**Get letter's position in alphabet (for lowercase letters only):**
```
XOR by backtick/chr(96)/binary('1100000')/hex('60') => (x ^ '`')
eg. ('d' ^ '`') => 4 ; ('x' ^ '`') => 24
```

## Miscellaneous

**Fast color conversion from R5G5B5 to R8G8B8 pixel format using shifts**
```
R8 = (R5 << 3) | (R5 >> 2)
G8 = (G5 << 3) | (G5 >> 2)
B8 = (B5 << 3) | (B5 >> 2)
```
Note: using anything other than the English letters will produce garbage results

## Switching Algebra 

**Read individual bits from an 8-bit register:**
> [!NOTE]
> Idea: Captures an 8-bit register value, and unmarshall it into individual bits based on a physical model for the conventional 8-bit register CMOS/TTL module. The trick is to combine the use of the logical Conjunction connective with the bitwise shifting to toggle individual bits. To better understand the algorithm, start solving the inner components first, then walk outwards out of the parentheses (e.g., `pregister->bit0 = (control_register & (1 << IEEE1284_PIN_0)) && IEEE1284_LOGIC_ON;` first, `(1 << IEEE1284_PIN_0)` selects the right bit by shifting a logical 1 to the bit location, then `control_register & bit_location` assigns the state for this bit by bitwise ANDing corresponding bits, and eventually assigns a logical state from the set of binary states `B = {0, 1}` using a logical AND operation.
> 
Example: 
```c
// Header definitions...
#define IEEE1284_LOGIC_ON ((uint8_t)(0xFF))
#define IEEE1284_LOGIC_OFF ((uint8_t)(0x00))

#define IEEE1284_PIN_0 ((uint8_t)0b0)
#define IEEE1284_PIN_1 ((uint8_t)0x01)
#define IEEE1284_PIN_2 ((uint8_t)0x02)
#define IEEE1284_PIN_3 ((uint8_t)0x03)
#define IEEE1284_PIN_4 ((uint8_t)0x04)
#define IEEE1284_PIN_5 ((uint8_t)0x05)
#define IEEE1284_PIN_6 ((uint8_t)0x06)
#define IEEE1284_PIN_7 ((uint8_t)0x07)
...
/**
 * @brief Defines a physical model for an 8-bit register.
 * this physical model holds a state from the set of logical
 * binary states, B = {0, 1}.
 *
 * @default values are ZERO.
 * 
 * @note Any condition in which the user passes a number larger than zero,
 * the algorithm converts it to 1, and otherwise zero.
 */
struct parport_register {
    uint8_t bit0;
    uint8_t bit1;
    uint8_t bit2;
    uint8_t bit3;
    uint8_t bit4;
    uint8_t bit5;
    uint8_t bit6;
    uint8_t bit7;
    uint8_t memory;
};
...
// source file definitions...
__int8_t pport_read_controls(parport_module *pmodule, parport_register *pregister) {
  if (pmodule == NULL || pmodule->fd < 0 || pregister == NULL) {
    return 1;
  }
  uint8_t control_register = 0x00;
  int value = ioctl(pmodule->fd, PPRCONTROL, &control_register);
  if (value < 0) {
    // leave the parport_register memory block without invoking any side effects!
    // TODO-Invoke on-failure callback processors
  } else {
    // write the control values
    // conversion of the bitwise values into logical values
    pregister->bit0 = (control_register & (1 << IEEE1284_PIN_0)) && IEEE1284_LOGIC_ON;
    pregister->bit1 = (control_register & (1 << IEEE1284_PIN_1)) && IEEE1284_LOGIC_ON;
    pregister->bit2 = (control_register & (1 << IEEE1284_PIN_2)) && IEEE1284_LOGIC_ON;
    pregister->bit3 = (control_register & (1 << IEEE1284_PIN_3)) && IEEE1284_LOGIC_ON;
    pregister->bit4 = (control_register & (1 << IEEE1284_PIN_4)) && IEEE1284_LOGIC_ON;
    pregister->bit5 = (control_register & (1 << IEEE1284_PIN_5)) && IEEE1284_LOGIC_ON;
    pregister->bit6 = (control_register & (1 << IEEE1284_PIN_6)) && IEEE1284_LOGIC_ON;
    pregister->bit7 = (control_register & (1 << IEEE1284_PIN_7)) && IEEE1284_LOGIC_ON;
    pregister->memory = control_register;
    // TODO-Invoke callback processors
  }
  return value;
}
```

**Write individual bits on an 8-bit register:**
> [!NOTE]
> Idea: Uses a physical model for an 8-bit register CMOS/TTL module to capture binary states for individual bits (aka. Flip Flops), then passes them to the target register in a _write commanded operation_.
> 
Example: 
```c
// Header definitions...
#define IEEE1284_LOGIC_ON ((uint8_t)(0xFF))
#define IEEE1284_LOGIC_OFF ((uint8_t)(0x00))

#define IEEE1284_PIN_0 ((uint8_t)0b0)
#define IEEE1284_PIN_1 ((uint8_t)0x01)
#define IEEE1284_PIN_2 ((uint8_t)0x02)
#define IEEE1284_PIN_3 ((uint8_t)0x03)
#define IEEE1284_PIN_4 ((uint8_t)0x04)
#define IEEE1284_PIN_5 ((uint8_t)0x05)
#define IEEE1284_PIN_6 ((uint8_t)0x06)
#define IEEE1284_PIN_7 ((uint8_t)0x07)
...
/**
 * @brief Defines a physical model for an 8-bit register.
 * this physical model holds a state from the set of logical
 * binary states, B = {0, 1}.
 *
 * @default values are ZERO.
 * 
 * @note Any condition in which the user passes a number larger than zero,
 * the algorithm converts it to 1, and otherwise zero.
 */
struct parport_register {
    uint8_t bit0;
    uint8_t bit1;
    uint8_t bit2;
    uint8_t bit3;
    uint8_t bit4;
    uint8_t bit5;
    uint8_t bit6;
    uint8_t bit7;
    uint8_t memory;
};
...
// source file definitions...
__int8_t pport_write_controls(parport_module *pmodule, parport_register *pregister) {
  if (pmodule == NULL || pmodule->fd < 0 || pregister == NULL) {
    return 1;
  }
  // conversion of the logical values to bitwise values
  // first: convert the user values into strict logic values (i.e., ON or OFF) (Logic ANDing).
  // second: bitwise left shift the converted logic values into its position in the register (Lshift).
  // third: add the positioned bits together to build the register (Bitwise ORing).
  // forth: write the byte to the control register (IO Commanding).
  pregister->bit0 = (pregister->bit0 && IEEE1284_LOGIC_ON) << IEEE1284_PIN_0;
  pregister->bit1 = (pregister->bit1 && IEEE1284_LOGIC_ON) << IEEE1284_PIN_1;
  pregister->bit2 = (pregister->bit2 && IEEE1284_LOGIC_ON) << IEEE1284_PIN_2;
  pregister->bit3 = (pregister->bit3 && IEEE1284_LOGIC_ON) << IEEE1284_PIN_3;
  pregister->bit4 = (pregister->bit4 && IEEE1284_LOGIC_ON) << IEEE1284_PIN_4;
  pregister->bit5 = (pregister->bit5 && IEEE1284_LOGIC_ON) << IEEE1284_PIN_5;
  pregister->bit6 = (pregister->bit6 && IEEE1284_LOGIC_ON) << IEEE1284_PIN_6;
  pregister->bit7 = (pregister->bit7 && IEEE1284_LOGIC_ON) << IEEE1284_PIN_7;
  // 
  pregister->memory = pregister->bit0 | pregister->bit1 | pregister->bit2 | pregister->bit3 |
                      pregister->bit4 | pregister->bit5 | pregister->bit6 | pregister->bit7;
  // write the data to the register
  int ret = ioctl(pmodule->fd, PPWCONTROL, &(pregister->memory));
  // TODO-invoke callback processors here...
  return ret;
}
```

**Equivalent operator of switching XOR:**
> [!NOTE]
> Idea: Brainstorming around combinatorial digital circuit design by providing the equivalent circuitry for the XOR.
> 
Example:
```c
// Header definitions...
#if !defined(SWITCHING_TYPE)
#define SWITCHING_TYPE uint8_t
#endif

// Source file definitions...
#include <electrostatic/electronetsoft/algorithm/arithmos/algebra/switching.h>
#include <stdlib.h>

uint8_t switching_xor(SWITCHING_TYPE **inputs, SWITCHING_TYPE *output){
    if (inputs == NULL || output == NULL) {
        return 1;
    }
    for (int i = 0; inputs[i] != NULL; i++) {
        SWITCHING_TYPE prop0 = *output;
        SWITCHING_TYPE prop1 = *(inputs[i]);
        // XOR Propositional operation break-down
        *output = ((!prop0) & prop1) | (prop0 & (!prop1));
        //
        // Example:
        // 0 ^ 1 ^ 1 = 0
        // -- break down -- 
        // 1. [(!(0) & 1) | (0 & !(1)] = 1
        // 2. [!(1) & 1) | (1 & !(1)] = 0
        //
        // XOR Function: Tests whether 2 binary sets are mutually exclusive
        // return 1 if the predicate holds, 0 otherwise.
        //
        // Note: If the negation of set A can intersect with set B OR the negation
        // of set B can intersect with set A; then, both sets are mutually exclusive.
        //
        // Note: This operates ONLY on binary sets (a set composed of only 0 or 1).
        //
    }
    return 0;
}
```

## Additional Resources

* [Bit Twiddling Hacks](https://graphics.stanford.edu/~seander/bithacks.html)
* [Floating Point Hacks](https://github.com/leegao/float-hacks)
* [Hacker's Delight](http://www.hackersdelight.org/)
* [The Bit Twiddler](http://bits.stephan-brumme.com/)
* [Bitwise Operations in C - Gamedev.net](https://www.gamedev.net/articles/programming/general-and-gameplay-programming/bitwise-operations-in-c-r1563/)
* [Bitwise Operators YouTube](https://www.youtube.com/results?search_query=bitwise+operators)
* [IEEE-1284 Module Library for parallel port programming - The ElectroKIO Project](https://github.com/Electrostat-Lab/Electrostatic-Sandbox/blob/master/electrostatic-sandbox-framework/electrostatic-core/src/libs/electrostatic-primer/electroio/electrokio/ieee1284_module.c)
* [Switching and Finite Automata Theory by Zvi Kohavi, Niraj K. Jha](https://www.amazon.com/Switching-Finite-Automata-Theory-Kohavi/dp/0521857481)
