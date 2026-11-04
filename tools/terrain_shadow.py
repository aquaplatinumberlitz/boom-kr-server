"""Compatibility import for the pure terrain candidate, not a runtime launcher."""
from pathlib import Path
import sys

# Keep direct offline scripts working without installing this repository.
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from server.terrain_model import Blast, Bomb, TerrainShadow  # noqa: E402,F401
