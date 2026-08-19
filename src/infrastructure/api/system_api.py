import os
import shutil
import subprocess
import tempfile


class SystemApi:
    def run_command(self, command: str, timeout: int = 30) -> tuple[str, str]:
        result = subprocess.run(
            command, shell=True, capture_output=True, text=True, timeout=timeout
        )
        return result.stdout, result.stderr

    def binary_exists(self, name: str) -> bool:
        return shutil.which(name) is not None

    def write_temp_file(self, content: str, suffix: str = "") -> str:
        fd, path = tempfile.mkstemp(suffix=suffix)
        os.close(fd)
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        return path

    def read_file(self, path: str) -> str:
        with open(path, "r", encoding="utf-8") as f:
            return f.read()

    def delete_file(self, path: str):
        if os.path.exists(path):
            os.unlink(path)
