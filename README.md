# Stellerpool — davetli tasarruf grubu prototipi

> Birlikte biriktir. Her şeyi doğrula. — *Save together. Verify everything.*

Stellerpool, altın günü gibi kapalı bir grubun sabit sıralı katkılarını Stellar üzerinde görünür kurallarla koordine etmeyi araştırır. Hedef Soroban kontratında kurucunun ortak parayı serbestçe çekme yetkisi olmayacak; üyeler sıra ve koşulları onaylayacak. Bu, erken teslim alan üyenin gelecek ödemelerini veya ev/araç teslimini garanti etmez.

Rise In x Stellar **Pro Hackathon 2026** için hazırlanıyor. Ayrıntılar: [ürün planı](docs/plan.md), [Mainnet tanıtım kararı](docs/mainnet-showcase.md), [hukuki sınır](docs/altin-gunu-legal-boundary.md) ve [AI denetimi/gerçek ürün yolu](docs/legal-ai-path.md).

## Bugünkü durum

**Testnet arayüzü** cüzdan ve ağ bağlantısını sağlar; havuz kontratı henüz yazılıp yayınlanmadığı için havuz, satıcı ödemesi ve iade işlemleri çalışmaz. **Mainnet tanıtım derlemesi** yalnızca canlı ağ defterini okur ve fikri anlatır; cüzdan imzası veya fon işlemi yapmaz. Anchor adımları yerel simülasyondur; gerçek TRY, token veya kullanılabilir bakiye üretmez. AI denetçisi henüz çalışan entegrasyon değildir.

Hackathon DOC'u gerçek TL yatırma/çekme karşılığında kullanılabilir Stellar bakiyesi istiyor. Seçilen anchor simülasyonu bu ölçütü karşılamaz; eksik kalem olarak sunulur.

## Hedef havuz kuralı

- Davetli, sabit üyeli grupta katkı, takvim ve sıra üyelerin aynı sürümü onaylamasıyla kilitlenir.
- Her turda herkes **kendi** katkısını yatırmadan o turun tahsisatı açılmaz. Ödeme gecikirse ek süre verilir; karşılanmazsa tur durdurulur.
- O tur henüz satıcıya ödenmediyse yalnızca **o turda yatırılan katkılar** iade edilir. Daha önce satıcıya gitmiş katkılar kontrattan geri alınamaz.
- Demo tahsisatı yalnızca izinli test satıcısına, alım önerisi ve insan doğrulayıcı onayı sonrasında gider. Alım için son tarih bulunur.
- Ayrı sponsor, avans veya platformun tam iade/teslimat garantisi yoktur. Fazla kullanıcı da erken teslim alanın gelecekteki ödeme riskini ortadan kaldırmaz.
- AI için hedef rol, belge ve koşul tutarsızlığını insanlara bildirmektir; fon anahtarı veya bağımsız ödeme/onay yetkisi yoktur.

**Risk örneği:** Dört üye 10'ar birim yatırıp ilk turda 40 birim A'nın satıcısına ödenirse havuzda ilk turun parası kalmaz. A sonraki turda ödemezse yeni tur durabilir; B, C ve D'nin ilk tur katkıları otomatik iade edilemez. [Ayrıntılı akış ve test değişmezleri](docs/plan.md).

Eminevim ve Fuzul'ün kamuya açık açıklamaları grup tasarrufu, sözleşmeli teslimat, şirket yükümlülüğü ve teslimat sonrası ipotek/rehin gibi araçları anlatır. Bizim kapalı grup demosunda lisanslı şirket, gerçek mülkiyet teminatı veya tahsilat hizmeti yoktur. Gerçek fon ve ev/araç finansmanı için uygun sözleşme, fon, ödeme ve yetkili taraflar ayrı kurulmalıdır. Cüzdan imzası veya AI raporu sözleşmeye kendiliğinden resmiyet vermez.

## Örnek akış

~~~mermaid
flowchart TD
  A[Havuz kurulur] --> B[Üyeler sırayı ve koşulları onaylar]
  B --> C[Her üye bu turun katkısını yatırır]
  C --> D{Tüm katkılar geldi mi?}
  D -->|Hayır, ek süre doldu| E[Bu turun katkıları iade edilir]
  D -->|Evet| F[Satıcı ve belge önerilir]
  F --> G{İnsan onayı ve süre uygun mu?}
  G -->|Hayır| E
  G -->|Evet| H[Bu turun tutarı demo satıcısına ödenir]
  H --> C
~~~

## Teknoloji ve çalıştırma

| Klasör | Amaç |
|---|---|
| `contracts/` | Planlanan Rust/Soroban kontratı ve testleri |
| `frontend/` | Vue 3, TypeScript, Stellar Wallets Kit arayüzü |
| `backend/` | Gerekirse zincir dışı doğrulama servisi |
| `scripts/` | Testnet kurulum yardımcıları |
| `docs/` | Ürün, hukuk ve demo kararları |

`frontend/` içinde `npm ci` çalıştırın. Testnet arayüzü için `npm run dev`; Mainnet'e bağlanan işlemsiz tanıtım derlemesi için `VITE_STELLAR_NETWORK=mainnet npm run build` kullanın. Statik çıktı `frontend/dist/` klasöründedir. Bu derleme Soroban kontratını Mainnet'e dağıtmaz. [Yayın kontrolü](docs/mainnet-showcase.md).

## Teslim durumu

- [x] Sponsorsuz tur, satıcı ödemesi ve yalnız mevcut tur iadesi değişmezleri test edilmiş Soroban kontratı (19 birim testi, [docs/IMPLEMENTATION_LOG.md](docs/IMPLEMENTATION_LOG.md) Faz 12)
- [x] Kuruluş/ödeme/alım son tarihleri, üye onayı ve geçmiş tur iade yasağı testleri
- [x] Testnet contract ID ve yeniden üretilebilir kurulum ([scripts/deploy_testnet.sh](scripts/deploy_testnet.sh); canlı kontrat `CCAKOEC34WVBKQ427KT5PI5GMPPKSGBHCWBI7FO4GNG247KKDUH67AZH`)
- [x] Mainnet için işlemsiz, canlı ağ bilgisini okuyan tanıtım derlemesi
- [ ] Kontrat durumlarıyla uyumlu cüzdanlı arayüz ve demo satıcı ödemesi (kontratın frontend'in `pool.ts`/`types/pool.ts` istemcisiyle birebir alan/fonksiyon eşleşmesi doğrulandı; cüzdan üzerinden tıklanabilir uçtan uca akış henüz denenmedi)
- [x] Eksik ödeme → ek süre → tur durması → mevcut tur iadesi demosu ([scripts/demo_testnet.sh](scripts/demo_testnet.sh), plan'ın kanonik senaryosu + canlı cure akışı Testnet'te çalıştırıldı)
- [x] Açıkça simülasyon etiketli anchor akışı
- [ ] Gerçek TL anchor giriş/çıkışı ve kullanılabilir bakiye kanıtı (hackathon şartı, mevcut kapsamda karşılanmıyor)
- [ ] AI denetçisi için gerekçeli rapor ve insan kararı
