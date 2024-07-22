import json
import sqlite3

def main():
    with open("src/resources/items.json", "r", encoding="utf-8") as f:
        items = json.load(f)
    conn = sqlite3.connect("sqlite.db")
    c = conn.cursor()

    for item in items:
        print(item)
        if item.get("en") == "" or item.get("de") == "" or item.get("ja") == "" or item.get("fr") == "":
            continue
        c.execute(f"INSERT INTO items VALUES (?,?,?,?,?)", (item["id"], item["en"], item["de"], item["ja"], item["fr"],))

    conn.commit()
    conn.close()

if __name__ == "__main__":
    main()
