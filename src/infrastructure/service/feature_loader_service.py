import importlib.util
import json
import os

from domain.entity.feature_entity import FeatureEntity


class FeatureLoaderService:
    def __init__(self, features_dir: str):
        self.features_dir = features_dir

    def get_sections(self) -> list[str]:
        sections = []
        for name in sorted(os.listdir(self.features_dir)):
            path = os.path.join(self.features_dir, name)
            if os.path.isdir(path) and not name.startswith("_"):
                sections.append(name)
        return sections

    def get_features_in_section(self, section: str) -> list[FeatureEntity]:
        section_dir = os.path.join(self.features_dir, section)
        features = []
        for name in sorted(os.listdir(section_dir)):
            feature_dir = os.path.join(section_dir, name)
            card_path = os.path.join(feature_dir, "card.json")
            feature_py = os.path.join(feature_dir, "feature.py")
            if (
                os.path.isdir(feature_dir)
                and os.path.exists(card_path)
                and os.path.exists(feature_py)
            ):
                with open(card_path) as f:
                    card = json.load(f)
                icon_svg = os.path.join(feature_dir, "icon.svg")
                icon_png = os.path.join(feature_dir, "icon.png")
                if os.path.exists(icon_svg):
                    icon_path = icon_svg
                elif os.path.exists(icon_png):
                    icon_path = icon_png
                else:
                    icon_path = None
                features.append(
                    FeatureEntity(
                        name=name,
                        description=card.get("description", ""),
                        section=section,
                        icon_path=icon_path,
                        folder_path=feature_dir,
                    )
                )
        return features

    def load_feature_instance(self, feature: FeatureEntity):
        feature_py = os.path.join(feature.folder_path, "feature.py")
        spec = importlib.util.spec_from_file_location(
            f"features.{feature.section}.{feature.name}", feature_py
        )
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module.Feature()
