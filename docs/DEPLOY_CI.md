# Otomatik dağıtım (GitHub Actions)

`main`'e `frontend/` altında bir değişiklik push'landığında
[`deploy-frontend.yml`](../.github/workflows/deploy-frontend.yml) siteyi derleyip
https://stellerpool.arslanyusuf.com adresine yükler. Elle de çalıştırılabilir (Actions sekmesi →
"Deploy frontend" → Run workflow).

## Ne yapar

1. `npm ci` + `npm run build` (tip kontrolü dahil; hata varsa dağıtım durur).
2. Derleme çıktısında Stellar gizli anahtarı deseni (`S…` + 55 karakter) taranır; bulunursa durur.
3. `rsync` ile sunucudaki `~/stellerpool/site` klasörüne yükler (`--delete`).
4. Yayındaki `index.html`, yeni derlemenin ana betiğini gösterene kadar bekler; göstermezse iş başarısız olur.

Yalnızca `frontend/**` ve workflow dosyası değişince tetiklenir. Yalnızca docs veya kontrat
değişikliği siteyi yeniden dağıtmaz.

## Güvenlik tasarımı

- Repo herkese açıktır, bu yüzden sunucu adresi, kullanıcı ve anahtar dosyaya **yazılmaz**, GitHub Secrets'ta durur.
- Dağıtım anahtarı sunucuda **yalnızca** `~/stellerpool/site` klasörüne yazabilir (`rrsync -wo`) ve
  `restrict` ile kısıtlıdır: kabuk, port yönlendirme, başka klasör ya da komut çalıştıramaz.
- Sunucunun host anahtarı sabitlenmiştir (`StrictHostKeyChecking=yes`), ortadaki adam saldırısına karşı.
- Workflow yalnızca `main`'e push ile ve elle tetiklenir; fork PR'ları Secret'lara erişemez.
- Aynı anda tek dağıtım çalışır (`concurrency`).

## Bir kerelik kurulum

**1. Sunucuda anahtarı yetkilendir** (sunucu sahibi, tek satır):

```bash
ssh contabo-yusuf 'cat >> ~/.ssh/authorized_keys' < authorized_keys_line.txt
```

`authorized_keys_line.txt` içeriği şu biçimdedir (ortak anahtar tek satırdır):

```text
restrict,command="/usr/bin/rrsync -wo /home/yusuf/stellerpool/site" ssh-ed25519 AAAA… stellerpool-github-actions-deploy
```

**2. GitHub Secrets'ı ekle** (repo sahibi): `DEPLOY_SSH_KEY` (özel anahtar), `DEPLOY_HOST`,
`DEPLOY_USER`, `DEPLOY_KNOWN_HOSTS` (host anahtarı satırı).

```bash
gh secret set DEPLOY_SSH_KEY       --repo yusufarslan44/Stellerpool < deploy_key
gh secret set DEPLOY_HOST          --repo yusufarslan44/Stellerpool < host.txt
gh secret set DEPLOY_USER          --repo yusufarslan44/Stellerpool < user.txt
gh secret set DEPLOY_KNOWN_HOSTS   --repo yusufarslan44/Stellerpool < known_hosts
```

Bittikten sonra yerel özel anahtar dosyasını sil.

## Yapılandırma değişkenleri (kod değişmeden site ayarı)

Derleme zamanı ayarları repo **Variables** ile verilir (Settings → Secrets and variables → Actions →
Variables). Tanımsızsa varsayılanlar geçerlidir.

| Değişken | Varsayılan | Ne zaman |
|---|---|---|
| `VITE_ROTATING_POOL_CONTRACT_ID` | güncel v10 kontratı | Yeni kontrat yayınlanınca |
| `VITE_MAX_MEMBERS` | `30` | Eski kontrata bağlanılırsa `12` |
| `VITE_ANCHOR_HOME_DOMAIN` | SDF test anchor'ı | Kendi anchor'ımız ayağa kalkınca |
| `VITE_POOL_ASSET_CODE` / `VITE_POOL_ASSET_ISSUER` | Testnet USDC | Havuz varlığı `TRYT` olunca |

Değişkeni değiştirdikten sonra siteye yansıması için workflow'u yeniden çalıştırmak gerekir (Run workflow).

## Elle geri alma

Sunucudaki dosyalar doğrudan yazılır, yedek tutulmaz. Sorunlu bir sürümü geri almak için önceki commit'e
dönüp (`git revert`) push'la ya da Actions'ta eski çalıştırmayı yeniden çalıştır.
