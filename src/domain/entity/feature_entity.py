from dataclasses import dataclass
from typing import Optional


@dataclass
class FeatureEntity:
    name: str
    description: str
    section: str
    icon_path: Optional[str]
    folder_path: str
