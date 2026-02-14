import urllib.request
import ssl

url = "https://fonts.loli.net/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200"
headers = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'}
context = ssl._create_unverified_context()

try:
    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, context=context) as response:
        content = response.read().decode('utf-8')
        print(content)
except Exception as e:
    print(e)
