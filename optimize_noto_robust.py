import os
import re
import urllib.request
import ssl
import shutil
import time
import hashlib

# Configuration
NOTO_CSS_URL = "https://fonts.loli.net/css2?family=Noto+Sans+SC:wght@400;500;700&display=swap"
NOTO_DIR = r"src/assets/fonts/NotoSansSC"
CSS_FILE = r"src/assets/fonts/fonts.css"
HEADERS = {
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    'Accept': 'text/css,*/*;q=0.1'
}

ssl._create_default_https_context = ssl._create_unverified_context

def download_file_with_retry(url, local_path, retries=3):
    if os.path.exists(local_path) and os.path.getsize(local_path) > 0:
        return True # Skip if exists
        
    for attempt in range(retries):
        try:
            req = urllib.request.Request(url, headers=HEADERS)
            with urllib.request.urlopen(req, timeout=10) as response, open(local_path, 'wb') as f:
                shutil.copyfileobj(response, f)
            print(f"Downloaded: {os.path.basename(local_path)}")
            return True
        except Exception as e:
            print(f"  Attempt {attempt+1}/{retries} failed for {url}: {e}")
            time.sleep(1) # wait before retry
            
    print(f"FAILED to download: {url}")
    return False

def optimize_noto():
    print("Starting Noto Sans SC optimization (Sliced)...")
    
    # 1. Fetch CSS to get slice URLs
    try:
        req = urllib.request.Request(NOTO_CSS_URL, headers=HEADERS)
        with urllib.request.urlopen(req) as response:
            css_content = response.read().decode('utf-8')
    except Exception as e:
        print(f"Error fetching CSS: {e}")
        return

    # Ensure/Clean Directory
    # Don't delete if we want to resume? Actually clearer to start fresh or just overwrite
    if not os.path.exists(NOTO_DIR):
        os.makedirs(NOTO_DIR)

    # 2. Parse and Download
    # Strategy: Replace long file references with short local names to avoid path limit issues
    # and download them.
    
    url_map = {} # remote_url -> local_filename
    
    def replacer(match):
        full_url = match.group(1)
        
        # Create a short filename based on content hash or just original name if short enough
        # The original goog filenames correspond to unicode ranges roughly.
        # But they are long. Let's use the hash of the URL to keep it consistent and short.
        # e.g., noto_sc_a1b2c3d4.woff2
        
        # Extract the extension
        ext = "woff2"
        if "ttf" in full_url: ext = "ttf"
        
        # Hash the URL to get a unique short name
        url_hash = hashlib.md5(full_url.encode('utf-8')).hexdigest()[:12]
        local_filename = f"slice_{url_hash}.{ext}"
        
        url_map[full_url] = local_filename
        return f"url('./NotoSansSC/{local_filename}')"

    # Replace URLs in CSS content
    new_css = re.sub(r'url\((https://[^)]+)\)', replacer, css_content)
    
    # 3. Download files in map
    print(f"Found {len(url_map)} font slices. Downloading...")
    success_count = 0
    fail_count = 0
    
    for url, filename in url_map.items():
        local_path = os.path.join(NOTO_DIR, filename)
        if download_file_with_retry(url, local_path):
            success_count += 1
        else:
            fail_count += 1
            
    print(f"Download finished. Success: {success_count}, Failed: {fail_count}")
    
    if fail_count > 0:
        print("WARNING: Some font slices failed to download. You may have missing characters.")
        return

    # 4. Merge into main fonts.css
    # We need to preserve the Material Symbols part of fonts.css and replace Noto part
    
    if os.path.exists(CSS_FILE):
        with open(CSS_FILE, 'r', encoding='utf-8') as f:
            current_full_css = f.read()
            
        # Extract Material Symbols part (assuming it is at the end or clearly marked)
        # Or simpler: Just keep the Material Symbols block we added earlier
        
        ms_block = ""
        ms_match = re.search(r'/\* Material Symbols Outlined - Full Variable Set \*/.*', current_full_css, re.DOTALL)
        if ms_match:
            ms_block = ms_match.group(0)
        else:
            # Fallback if we can't find marker, try finding the class definition
            if ".material-symbols-outlined" in current_full_css:
                # This is risky, manual reconstruction is safer if we know what we want.
                ms_block = """
/* Material Symbols Outlined - Full Variable Set */
@font-face {
  font-family: 'Material Symbols Outlined';
  font-style: normal;
  font-weight: 100 700;
  src: url('./MaterialSymbolsOutlined/MaterialSymbolsOutlined_Full.woff2') format('woff2');
}

.material-symbols-outlined {
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
}
"""
        
        # Combine
        final_css = new_css + "\n" + ms_block
        
        # Remove old @font-face for Noto if any (the new_css replaces them entirely)
        # Actually new_css ONLY contains the @font-face rules from the Noto URL.
        # So it's perfect.
        
        with open(CSS_FILE, 'w', encoding='utf-8') as f:
            f.write(final_css)
            
        print("fonts.css updated.")

if __name__ == "__main__":
    optimize_noto()
