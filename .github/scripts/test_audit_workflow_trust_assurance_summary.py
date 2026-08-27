#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

TARGET = Path(__file__).with_name("test_build_af01_assurance_summary.py")
SPEC = importlib.util.spec_from_file_location("af01_assurance_summary_tests", TARGET)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)

AssuranceSummaryTests = MODULE.AssuranceSummaryTests
