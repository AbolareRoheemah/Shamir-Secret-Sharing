# Shamir's Secret Sharing

This project implements Shamir's Secret Sharing scheme using polynomial interpolation. It allows you to share a secret among a group of participants, where only a subset of them can reconstruct the secret.

## Overview

Shamir's Secret Sharing is a cryptographic algorithm that divides a secret into multiple parts (shares) such that only a certain number of shares (threshold) are needed to reconstruct the secret. This is achieved by creating a polynomial of degree \( k-1 \) (where \( k \) is the threshold) and evaluating it at different points.

## Features

- Generate random coefficients for the polynomial.
- Share a secret among a specified number of participants.
- Recover the secret using a subset of shares.
- Modular arithmetic to ensure operations are performed within a finite field.

## Dependencies

This project uses the following Rust libraries:

- `num-bigint`: For handling large integers.
- `num-traits`: For numeric traits.
- `lazy_static`: For defining static variables.
- `rand`: For generating random numbers.
- `ark-ff` and `ark-bn254`: For finite field arithmetic.

## Installation

To run this project, ensure you have Rust installed. You can clone this repository and build it using Cargo:
```bash
git clone <repository-url>
cd shamir-secret-sharing
cargo build
```

## Usage

You can run the example in the `main.rs` file to see how the secret sharing works:

```bash
cargo run
```

This will generate shares for a given secret and demonstrate the recovery process.

## Tests

To run the tests included in the project, use the following command:

```bash
cargo test
```

## Contributing

Contributions are welcome! If you have suggestions for improvements or new features, feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
