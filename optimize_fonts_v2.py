import os
import re
import urllib.request
import ssl
import shutil
import time
# 1. Configuration
# Material Symbols: Must use Google to guarantee Variable WOFF2 format (Mirrors often serve static TTF)
CSS_URL = "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200"
FONTS_DIR = "src/assets/fonts/MaterialSymbolsOutlined"

# Noto Sans SC: Use Mirror for speed (Sliced WOFF2)
NOTO_CSS_URL = "https://fonts.loli.net/css2?family=Noto+Sans+SC:wght@400;500;700&display=swap"
NOTO_DIR = "src/assets/fonts/NotoSansSC"

CSS_FILE = "src/assets/fonts/fonts.css"
HEADERS = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'}

ssl._create_default_https_context = ssl._create_unverified_context

def download_file(url, local_path):
    req = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(req) as response, open(local_path, 'wb') as f:
        shutil.copyfileobj(response, f)

def update_fonts_css_simple_full():
    print("Starting optimized font consolidation...")

    # 1. Download Material Symbols Full Variable Font
    print("Fetching Material Symbols Full Variable Font info...")
    try:
        req = urllib.request.Request(CSS_URL, headers=HEADERS)
        with urllib.request.urlopen(req) as response:
            css_content = response.read().decode('utf-8')
        
        # Extract woff2 url
        match = re.search(r'url\((https://[^)]+)\) format\(\'woff2\'\)', css_content)
        if match:
            font_url = match.group(1)
            filename = "MaterialSymbolsOutlined_Full.woff2"
            local_path = os.path.join(FONTS_DIR, filename)
            
            # Clean directory first but keep parent structure
            if os.path.exists(FONTS_DIR):
                shutil.rmtree(FONTS_DIR)
            os.makedirs(FONTS_DIR, exist_ok=True)
            
            print(f"Downloading {filename} from {font_url}...")
            download_file(font_url, local_path)
            
            # Verify file size
            size_mb = os.path.getsize(local_path) / (1024 * 1024)
            print(f"Downloaded size: {size_mb:.2f} MB")
        else:
            print("Error: Could not find woff2 URL in Google Fonts CSS response.")
            # Don't return, continue to Noto

    except Exception as e:
        print(f"Warning: Material Symbols download failed ({e}). You may need to download it manually.")
        print("Continuing to process Noto Sans SC...")
        # Continue execution

    # 2. Re-create fonts.css
    # We will keep Noto Sans definitions intact? No, user reverted state, 
    # so currently fonts.css has the OLD TTF definitions for Noto and Material.
    # We need to:
    # A) Keep Noto Sans SC as is (or if user wants sliced woff2 optimization for Noto? 
    #    User said: "Noto Sans SC 使用单独 的小文件". 
    #    Wait, in the reverted state (commit cd5a619), does Noto use sliced small files?
    #    Let's check the current CSS content I just read.
    #    THE CURRENT CSS (viewed in Step 434) shows Noto using LOCAL TTF FILES: ./NotoSansSC/NotoSansSC-400.ttf etc.
    #    It does NOT use sliced woff2 files in this reverted state.
    #    But user request says: "Noto Sans SC 使用单独 的小文件" (Use separate small files).
    #    This implies I should ALSO re-download the Noto Sans SC sliced woff2 files 
    #    to replace the current big TTF files, as part of "optimization".
    
    print("Processing Noto Sans SC (Switching to sliced WOFF2 for optimization)...")
    
    # helper to download noto slices
    try:
        req = urllib.request.Request(NOTO_CSS_URL, headers=HEADERS)
        with urllib.request.urlopen(req) as response:
            noto_css_content = response.read().decode('utf-8')
            
        # Clean Noto dir
        if os.path.exists(NOTO_DIR):
            shutil.rmtree(NOTO_DIR)
        os.makedirs(NOTO_DIR, exist_ok=True)
        
        # Parse all font-face blocks
        # We need to extract src url and unicode-range for each block
        # And replace url with local path
        
        # Simple parsing strategy:
        # iterate line by line to build the new css content
        
        final_css_content = "/* Noto Sans SC - Localized Sliced WOFF2 */\n"
        
        # Download all referenced woff2 files
        urls = re.findall(r'url\((https://[^)]+)\)', noto_css_content)
        unique_urls = set(urls)
        
        url_map = {} # remote_url -> local_relative_path
        
        print(f"Downloading {len(unique_urls)} Noto Sans SC slice files...")
        
        for i, url in enumerate(unique_urls):
            # Generate a consistent short name
            # e.g., NotoSansSC-slice-0.woff2
            # Actually, let's keep part of hash to avoid collision if any, or just index
            # Index is safest if mapping is preserved
            
            # Wait, regex finding order matters for mapping back to CSS blocks.
            # Let's simple-replace.
            pass

        # Better strategy: modify the CSS string directly
        
        def replace_url(match):
            remote_url = match.group(1)
            filename = remote_url.split('/')[-1]
            # sanitize filename
            filename = re.sub(r'[^\w\-\.]', '_', filename)
            local_path = os.path.join(NOTO_DIR, filename)
            
            if not os.path.exists(local_path):
                # print(f"Downloading {filename}...")
                download_file(remote_url, local_path)
            
            return f"url('./NotoSansSC/{filename}')"

        new_noto_css = re.sub(r'url\((https://[^)]+)\)', replace_url, noto_css_content)
        final_css_content += new_noto_css
        
    except Exception as e:
        print(f"Error processing Noto Sans SC: {e}")
        return

    # 3. Add Material Symbols Block
    ms_css_block = f"""
/* Material Symbols Outlined - Full Variable Set */
@font-face {{
  font-family: 'Material Symbols Outlined';
  font-style: normal;
  font-weight: 100 700;
  src: url('./MaterialSymbolsOutlined/MaterialSymbolsOutlined_Full.woff2') format('woff2');
}}

.material-symbols-outlined {{
  font-family: 'Material Symbols Outlined';
  font-weight: normal;
  font-style: normal;
  font-size: 24px;
  line-height: 1;
  letter-spacing: normal;
  text-transform: none;
  display: inline-block;
  white-space: nowrap;
  word-wrap: normal;
  direction: ltr;
  /* NO extra smoothing to keep it bold/sharp as requested */
}}
"""
    final_css_content += "\n" + ms_css_block
    
    # Write fonts.css
    with open(CSS_FILE, 'w', encoding='utf-8') as f:
        f.write(final_css_content)
    
    print("Success! Fonts optimized:")
    print("1. Material Symbols: Merged into single variable woff2 (for size reduction)")
    print("2. Noto Sans SC: Converted to sliced woff2 (for load performance)")
    print("3. Rendering: Kept default (bold/sharp) style.")

if __name__ == "__main__":
    update_fonts_css_simple_full()
