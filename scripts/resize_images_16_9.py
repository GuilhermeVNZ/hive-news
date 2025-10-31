#!/usr/bin/env python3
# Script to resize all images in News-main/images to 16:9 aspect ratio
# Uses intelligent cropping (center-crop) to preserve image quality

import os
from pathlib import Path
from PIL import Image
import sys

def resize_to_16_9(image_path, output_size=(1920, 1080)):
    # Open image
    img = Image.open(image_path)
    original_width, original_height = img.size
    
    # Calculate 16:9 ratio
    target_ratio = 16 / 9
    img_ratio = original_width / original_height
    
    width, height = output_size
    
    # If image is already 16:9 or close, resize directly
    if abs(img_ratio - target_ratio) < 0.01:
        return img.resize((width, height), Image.Resampling.LANCZOS)
    
    # Calculate crop region to get 16:9 from center
    if img_ratio > target_ratio:
        # Image is wider - crop width
        new_width = int(original_height * target_ratio)
        left = (original_width - new_width) // 2
        crop_box = (left, 0, left + new_width, original_height)
    else:
        # Image is taller - crop height
        new_height = int(original_width / target_ratio)
        top = (original_height - new_height) // 2
        crop_box = (0, top, original_width, top + new_height)
    
    # Crop to 16:9
    cropped = img.crop(crop_box)
    
    # Resize to target size
    resized = cropped.resize((width, height), Image.Resampling.LANCZOS)
    
    return resized

def process_directory(base_path):
    base = Path(base_path)
    
    if not base.exists():
        print(f"Directory not found: {base_path}")
        return
    
    total_images = 0
    processed_images = 0
    
    # Supported image formats
    extensions = ['.jpg', '.jpeg', '.png', '.webp']
    
    # Process each category folder
    for category_folder in sorted(base.iterdir()):
        if not category_folder.is_dir():
            continue
        
        category = category_folder.name
        print(f"\nProcessing category: {category}")
        
        # Process each image in the category
        images = [f for f in category_folder.iterdir() 
                  if f.suffix.lower() in extensions]
        
        if not images:
            print(f"  No images found")
            continue
        
        print(f"  Found {len(images)} images")
        
        for img_path in sorted(images):
            try:
                total_images += 1
                
                print(f"  Processing: {img_path.name}...", end=" ")
                
                # Resize image
                resized_img = resize_to_16_9(img_path)
                
                # Get original format
                original_format = img_path.suffix[1:].upper()
                if original_format == 'JPG':
                    original_format = 'JPEG'
                
                # Save resized image (overwrite original)
                resized_img.save(img_path, format=original_format, quality=90)
                
                processed_images += 1
                print(f"OK")
                
            except Exception as e:
                print(f"ERROR: {e}")
    
    print(f"\n{'='*60}")
    print(f"Summary:")
    print(f"  Total images: {total_images}")
    print(f"  Successfully processed: {processed_images}")
    print(f"  Errors: {total_images - processed_images}")
    print(f"{'='*60}")

def main():
    base_path = "G:/Hive-Hub/News-main/images"
    
    print("Image Resizer - 16:9 Aspect Ratio")
    print("=" * 60)
    print(f"Target size: 1920x1080 (16:9)")
    print(f"Location: {base_path}")
    print("=" * 60)
    
    # Ask for confirmation
    response = input("\nThis will overwrite all existing images. Continue? (yes/no): ")
    if response.lower() != 'yes':
        print("Operation cancelled.")
        return
    
    # Process all images
    process_directory(base_path)
    
    print("\nDone!")

if __name__ == "__main__":
    main()
