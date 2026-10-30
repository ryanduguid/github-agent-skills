"""Local manifests discover the canonical five skills without adding runtime tools."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read_manifest(path):
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


class PluginPackagingTests(unittest.TestCase):
    def test_both_plugins_use_the_same_identity_and_canonical_source(self):
        claude = read_manifest(".claude-plugin/plugin.json")
        codex = read_manifest(".codex-plugin/plugin.json")
        for field in ("name", "version", "license", "repository", "author"):
            self.assertEqual(claude[field], codex[field], field)
        for plugin in (claude, codex):
            self.assertEqual(plugin["name"], "github-agent-skills")
            self.assertEqual(plugin["license"], "FSL-1.1-ALv2")
            self.assertEqual(plugin["skills"].rstrip("/"), "./.claude/skills")
            self.assertTrue((ROOT / plugin["skills"]).is_dir())
            self.assertNotIn("mcpServers", plugin)

    def test_marketplace_lists_every_canonical_skill_once(self):
        marketplace = read_manifest(".claude-plugin/marketplace.json")
        self.assertEqual(len(marketplace["plugins"]), 1)
        plugin = marketplace["plugins"][0]
        self.assertEqual(plugin["name"], "github-agent-skills")
        self.assertEqual(plugin["source"], "./")
        self.assertIs(plugin["strict"], True)
        self.assertEqual(plugin["license"], read_manifest(".claude-plugin/plugin.json")["license"])
        discovered = sorted(
            "./" + path.parent.relative_to(ROOT).as_posix()
            for path in (ROOT / ".claude/skills").glob("*/SKILL.md")
        )
        self.assertEqual(len(discovered), 5)
        self.assertEqual(sorted(plugin["skills"]), discovered)

    def test_codex_marketplace_is_local_and_skill_copies_match(self):
        marketplace = read_manifest(".agents/plugins/marketplace.json")
        self.assertEqual(len(marketplace["plugins"]), 1)
        plugin = marketplace["plugins"][0]
        self.assertEqual(plugin["name"], "github-agent-skills")
        self.assertEqual(plugin["source"], {"source": "local", "path": "./"})
        for source in (ROOT / ".claude/skills").rglob("*"):
            if source.is_file():
                generated = ROOT / ".agents/skills" / source.relative_to(ROOT / ".claude/skills")
                self.assertEqual(source.read_bytes(), generated.read_bytes(), source.name)


if __name__ == "__main__":
    unittest.main()
