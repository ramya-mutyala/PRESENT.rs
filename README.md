# PRESENT.rs

[![Build Status](https://travis-ci.org/yi-jiayu/PRESENT.rs.svg?branch=master)](https://travis-ci.org/yi-jiayu/PRESENT.rs)
[![codecov](https://codecov.io/gh/yi-jiayu/PRESENT.rs/branch/master/graph/badge.svg)](https://codecov.io/gh/yi-jiayu/PRESENT.rs)

Rust implementation of the PRESENT ultra-lightweight block cipher [1]

## Usage
### API
Coming soon!

### Command-line
Encrypt with an 80-bit key: 
```
present --key 0000000000 Tux.ppm > Tux.enc
```

Decrypt with the same key: 
```
present --key 0000000000 -d Tux.enc > Tux.dec.ppm
```

## References
[1] Bogdanov, A., Knudsen, L. R., Leander, G., Paar, C., Poschmann, A., Robshaw, M. J., ... & Vikkelsoe, C. (2007, September). PRESENT: An ultra-lightweight block cipher. In International Workshop on Cryptographic Hardware and Embedded Systems (pp. 450-466). Springer, Berlin, Heidelberg. ([PDF](https://www.iacr.org/archive/ches2007/47270450/47270450.pdf))
