#!/usr/bin/env python3
import os
import shutil
from datetime import datetime

PROJECT_ROOT = os.path.dirname(os.path.abspath(__file__))
DB_PATH = os.path.join(PROJECT_ROOT, "data", "trello.db")
BACKUPS_DIR = os.path.join(PROJECT_ROOT, "backups")

def main():
    if not os.path.exists(DB_PATH):
        print("❌ База данных не найдена:", DB_PATH)
        return

    os.makedirs(BACKUPS_DIR, exist_ok=True)
    timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    backup_path = os.path.join(BACKUPS_DIR, f"trello_{timestamp}.db")

    try:
        shutil.copy2(DB_PATH, backup_path)
        print(f"✅ Резервная копия создана: {backup_path}")
    except Exception as e:
        print(f"❌ Ошибка: {e}")

if __name__ == "__main__":
    main()