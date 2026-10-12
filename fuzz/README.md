The Atheris target exercises the public-file scanner with fabricated bytes:
BOM-marked and NUL-bearing text decoding, Quick start command extraction and
every safety pattern. It checks that decoding returns text or a failure, never
both, that extracted commands are stripped and non-empty, and that each pattern
finishes on arbitrary input. Unexpected exceptions and failed invariants fail
the run.

On Linux with Python 3.14, run:

```sh
python -m pip install --require-hashes --only-binary :all: -r requirements-fuzz.txt
python -m fuzz.public_files fuzz/corpus -max_total_time=60 -max_len=4096 -timeout=2 -rss_limit_mb=1024
```

CI runs the target after main pushes, weekly and on demand, with read-only
repository access and a five-minute timeout. The unittest suite replays the
checked-in corpus on every supported platform. Atheris does not support
Windows, so coverage-guided fuzzing runs on Linux. A successful run is bounded
evidence, not proof that every input is handled.

After changing the pinned version, regenerate its hashes with:

```sh
uv pip compile requirements-fuzz.in --python-version 3.14 --python-platform x86_64-unknown-linux-gnu --generate-hashes --output-file requirements-fuzz.txt
```
