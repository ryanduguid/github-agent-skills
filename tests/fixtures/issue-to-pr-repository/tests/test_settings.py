import unittest

from src.settings import load_settings


class LoadSettingsTests(unittest.TestCase):
    def test_loads_valid_json(self) -> None:
        self.assertEqual({"enabled": True}, load_settings('{"enabled": true}'))

    def test_keeps_json_error_for_nonempty_invalid_input(self) -> None:
        with self.assertRaises(ValueError):
            load_settings("not json")
