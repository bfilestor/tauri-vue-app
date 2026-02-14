import re
import os
import urllib.request
import ssl

# Create unverified context to avoid potential SSL errors
ssl._create_default_https_context = ssl._create_unverified_context

# Process Material Symbols
ms_css_path = 'src/assets/fonts/MaterialSymbolsOutlined.css'
ms_dir = 'src/assets/fonts/MaterialSymbolsOutlined'
os.makedirs(ms_dir, exist_ok=True)

with open(ms_css_path, 'r', encoding='utf-8') as f:
    ms_css_content = f.read()

# Pattern to find URLs
# src: url(https://...) ...
url_pattern = re.compile(r'url\((https://[^)]+)\)')

def replace_url(match):
    url = match.group(1)
    filename = url.split('/')[-1]
    local_path = os.path.join(ms_dir, filename)
    
    if not os.path.exists(local_path):
        print(f"Downloading {url} to {local_path}")
        try:
            # Use a custom opener to mimic a browser if needed, though simple retrieve often works
            opener = urllib.request.build_opener()
            opener.addheaders = [('User-agent', 'Mozilla/5.0')]
            urllib.request.install_opener(opener)
            urllib.request.urlretrieve(url, local_path)
        except Exception as e:
            print(f"Failed to download {url}: {e}")
            
    # Return relative path for CSS
    return f"url('./MaterialSymbolsOutlined/{filename}')"

new_ms_content = url_pattern.sub(replace_url, ms_css_content)

# Noto Sans SC part
# I already downloaded to src/assets/fonts/NotoSansSC/NotoSansSC-300.ttf etc.
noto_css = """
@font-face {
  font-family: 'Noto Sans SC';
  font-style: normal;
  font-weight: 300;
  font-display: swap;
  src: url('./NotoSansSC/NotoSansSC-300.ttf') format('truetype');
}
@font-face {
  font-family: 'Noto Sans SC';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('./NotoSansSC/NotoSansSC-400.ttf') format('truetype');
}
@font-face {
  font-family: 'Noto Sans SC';
  font-style: normal;
  font-weight: 500;
  font-display: swap;
  src: url('./NotoSansSC/NotoSansSC-500.ttf') format('truetype');
}
@font-face {
  font-family: 'Noto Sans SC';
  font-style: normal;
  font-weight: 700;
  font-display: swap;
  src: url('./NotoSansSC/NotoSansSC-700.ttf') format('truetype');
}
"""

final_css = noto_css + "\n" + new_ms_content

with open('src/assets/fonts/fonts.css', 'w', encoding='utf-8') as f:
    f.write(final_css)

print("Done processing fonts.")
