import json

def main():
    with open("src/resources/items.json", "r", encoding="utf-8") as f:
        items = json.load(f)
    
    converted_items = []
    for key,value in items.items():
        converted_items.append(
            {
                "id": int(key),
                **value
            })

    with open("src/resources/items.json", "w", encoding="utf-8") as f:
        json.dump(converted_items, f, indent=0, ensure_ascii=False, separators=(',', ':'))

if __name__ == "__main__":
    main()