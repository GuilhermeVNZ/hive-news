from PIL import Image
from pathlib import Path

base = Path("G:/Hive-Hub/News-main/images")

print("Verificação de Dimensões 16:9")
print("=" * 60)

for cat_folder in sorted(base.iterdir()):
    if not cat_folder.is_dir():
        continue
    
    category = cat_folder.name
    imgs = list(cat_folder.glob("*.jpg"))[:3]
    
    if imgs:
        print(f"\n{category}:")
        for img_path in imgs:
            img = Image.open(img_path)
            w, h = img.size
            ratio = w / h
            target_ratio = 16 / 9
            diff = abs(ratio - target_ratio)
            status = "OK" if diff < 0.01 else "WRONG"
            print(f"  {img_path.name}: {w}x{h} (ratio: {ratio:.4f}) {status}")





