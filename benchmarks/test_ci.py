from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

import ci


class WorkflowInputs(unittest.TestCase):
    def test_target_is_independent_of_the_baseline_and_workflow_branch(self):
        with patch.object(ci, "resolve_ref", side_effect=lambda ref: ref):
            args = ci.arguments({"BENCHMARK_REF": "abc123", "BASELINE_REF": "v1.1.12"}, Path("out.json"))
        self.assertEqual([args[i + 1] for i, arg in enumerate(args) if arg == "--ref"], ["v1.1.12", "abc123"])
        self.assertIn("--no-default-features", args)

    def test_single_version_and_explicit_features(self):
        with patch.object(ci, "resolve_ref", side_effect=lambda ref: ref):
            args = ci.arguments({"BENCHMARK_REF": "HEAD", "BASELINE_REF": "",
                                 "DEFAULT_FEATURES": "true", "BENCHMARK_FEATURES": "wasi,mruby-hash-fnv",
                                 "BENCHMARK_CASES": "cart, html_render", "BENCHMARK_MODE": "warm"}, Path("out.json"))
        self.assertEqual(args.count("--ref"), 1)
        self.assertNotIn("--no-default-features", args)
        self.assertIn("--features=wasi,mruby-hash-fnv", args)
        self.assertIn("--case=html_render", args)

    def test_invalid_measurement_settings(self):
        for inputs in [{"BENCHMARK_SAMPLES": "0"}, {"BENCHMARK_SAMPLES": "101"},
                       {"BENCHMARK_SAMPLES": "1.5"}, {"BENCHMARK_MODE": "invalid"},
                       {"DEFAULT_FEATURES": "invalid"}]:
            with self.subTest(inputs=inputs), patch.object(ci, "resolve_ref", return_value="HEAD"):
                with self.assertRaises(ValueError):
                    ci.arguments(inputs, Path("out.json"))

    def test_version_and_remote_branch_resolution(self):
        for requested, resolved in [("1.1.12", "v1.1.12"), ("topic", "origin/topic")]:
            def fake_run(args, **kwargs):
                success = args[-1] == resolved + "^{commit}"
                return subprocess.CompletedProcess(args, 0 if success else 128, "a" * 40 + "\n", "")
            with self.subTest(requested=requested), patch.object(ci.subprocess, "run", side_effect=fake_run):
                self.assertEqual(ci.resolve_ref(requested), "a" * 40)

    def test_invalid_refs_are_not_silently_replaced_with_head(self):
        with patch.object(ci.subprocess, "run", return_value=subprocess.CompletedProcess([], 128, "", "")):
            for ref in ("", "working-tree", "unknown-ref"):
                with self.subTest(ref=ref), self.assertRaises(ValueError):
                    ci.resolve_ref(ref)


if __name__ == "__main__":
    unittest.main()
