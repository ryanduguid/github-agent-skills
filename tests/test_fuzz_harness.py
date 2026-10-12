import unittest
from pathlib import Path
from unittest import mock

from fuzz import public_files

CORPUS = Path(__file__).resolve().parents[1] / "fuzz" / "corpus"


class FuzzHarnessTests(unittest.TestCase):
    def test_seed_corpus(self):
        seeds = sorted(CORPUS.iterdir())
        self.assertGreater(len(seeds), 0)
        for seed in seeds:
            with self.subTest(seed=seed.name):
                public_files.test_one_input(seed.read_bytes())

    def test_a_decoder_returning_text_and_failure_is_caught(self):
        with mock.patch.object(public_files.public_files, "decoded_text", return_value=("text", "undecodable-text")):
            with self.assertRaises(AssertionError):
                public_files.test_one_input(b"x")

    def test_unexpected_scanner_errors_escape(self):
        with mock.patch.object(public_files.public_files, "quick_start_commands", side_effect=RuntimeError("bug")):
            with self.assertRaisesRegex(RuntimeError, "bug"):
                public_files.test_one_input(b"x")


if __name__ == "__main__":
    unittest.main()
