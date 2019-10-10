#!/usr/bin/env bash

set -e

FLAGS="--release -- -n 16"

# Run 3 rounds and the beacon
rm -f challenge response

cargo run --bin new ${FLAGS}
# Wrote a fresh accumulator to `./challenge`

# Round 1

echo random1 | cargo run --bin compute ${FLAGS}
# Your contribution has been written to `./response`

cargo run --bin verify_transform ${FLAGS}
# Verification succeeded! Writing to `./new_challenge`

mv challenge challenge.0
mv response response.1
mv new_challenge challenge

# Round 2

echo random2 | cargo run --bin compute ${FLAGS} --digest response.2.digest
# Your contribution has been written to `./response`

cargo run --bin verify_transform ${FLAGS}
# Verification succeeded! Writing to `./new_challenge`

mv challenge challenge.1
mv response response.2
mv new_challenge challenge

# Round 3

echo random3 | cargo run --bin compute ${FLAGS}
# Your contribution has been written to `./response`

cargo run --bin verify_transform ${FLAGS}
# Verification succeeded! Writing to `./new_challenge`

mv challenge challenge.2
mv response response.3
mv new_challenge challenge

# Round 4 (Beacon)

echo random4 | cargo run --bin compute ${FLAGS}
# Your contribution has been written to `./response`

cargo run --bin verify_transform ${FLAGS}
# Verification succeeded! Writing to `./new_challenge`

mv challenge challenge.3
mv response response.4.beacon
mv new_challenge challenge

echo Creating transcript ...
rm -f transcript
for f in response.1 response.2 response.3 response.4.beacon ; do
    dd if=$f bs=64 skip=1 >> transcript
done

echo Verifying transcript ...
cargo run --bin verify ${FLAGS} -r 4 --skip-lagrange

echo Verifying transcript contains contribution ...
cargo run --bin verify ${FLAGS} -r 4 --skip-lagrange --digest response.2.digest

echo Verifying transcript does not contain invalid contribution ...
sed -e 's/0/1/g' response.2.digest > response.2.invalid.digest
if (cargo run --bin verify ${FLAGS} -r 4 --skip-lagrange --digest response.2.invalid.digest) ; then
    echo Verification failed
    exit 1
fi
