# Mainnet tanıtım sürümü ve Testnet demo sınırı

**Karar tarihi:** 19 Eylül 2026. Bu aşamada lisans başvurusu, gerçek müşteri fonu ve canlı TRY anchor yapılmayacak. Mainnet'e bağlanan bir tanıtım sitesi yayımlanabilir; havuz işlemleri yalnızca Testnet için geliştirilecek. Bu belge dağıtım sırasında gerçekte çalışan parçaları açıkça belirtir.

## Şu anda çalışanlar

| Parça | Durum | Kullanıcıya söylenecek ifade |
|---|---|---|
| Mainnet tanıtım derlemesi | Stellar Mainnet RPC ağ kimliğini doğrular ve son defter numarasını okur. Ürün fikrini gösterir. | “Mainnet ağ verisini okuyan işlemsiz tanıtım.” |
| Mainnet havuz işlemleri | Kapalı: havuz sayfaları, cüzdan bağlantısı, imza, Friendbot, trustline ve kontrat istemcisi erişimi yok. Mainnet kontratı yayımlanmadı. | “Mainnet'te havuz, fon yatırma ve ödeme bulunmuyor.” |
| Testnet arayüzü | Cüzdan, ağ ve test varlığı hazırlığı için kod var. Havuz kontratı henüz bulunmadığından havuz çağrıları çalışmaz. | “Testnet prototipi; havuz kontratı henüz yapılandırılmadı.” |
| Anchor | TRY yatırma/çekme adımlarını gösteren etkileşimli simülasyon var. Tutar veya kişisel bilgi alınmaz; API, imza, ödeme ve bakiye değişikliği yoktur. | “Anchor akış simülasyonu; gerçek para ve kullanılabilir bakiye yok.” |
| AI denetçisi | Tasarım belgelendi; çalışan model/karar akışı yok. | “Planlanan yardımcı denetim; resmî onay vermez.” |

Mainnet verisi okunması **Soroban kontratının Mainnet'e dağıtıldığı** anlamına gelmez. Bir statik web sitesinin internette yayımlanması da zincir üstü dağıtım değildir.

## Tanıtım sitesini derleme

`frontend/` klasöründe:

```sh
npm ci
VITE_STELLAR_NETWORK=mainnet npm run build
```

Çıktı `frontend/dist/` klasörüdür. Statik barındırma hizmeti kök dizini bu klasöre, bilinmeyen yollar için SPA fallback'i `index.html` dosyasına yönlendirilir. Barındırma sağlayıcısı henüz seçilmedi; bu nedenle canlı URL yok. `VITE_STELLAR_RPC_URL` boş bırakılırsa Mainnet varsayılan RPC adresi kullanılır. Özel adres verilirse uygulama ağ passphrase'ini denetler; yanlış ağ Mainnet defter numarası diye gösterilmez.

**Yayın öncesi kontrol:** Sayfada “işlemsiz tanıtım” uyarısı, cüzdan ve havuz butonlarının yokluğu, gerçek para/teslim garantisi bulunmadığı ifadesi ve `Mainnet havuz kontratı yayınlanmadı` durumu görülmelidir. `VITE_STELLAR_NETWORK=testnet` ile üretilmiş `dist/` yanlışlıkla Mainnet tanıtım sitesi olarak yüklenmemelidir. Önce Mainnet derlemesini yeniden çalıştırıp bu kontrol yapılır. Ortam değişkenleri Vite'ta derleme sırasında uygulanır; barındırma panelinde sonradan değiştirilmeleri mevcut statik dosyayı değiştirmez.

## Testnet ve anchor demosu için kabul ölçütleri

1. Soroban havuz kodu yazılır; yetkilendirme, ayrık muhasebe, gecikme, iptal, iade ve süre/TTL davranışı anlamlı testlerle doğrulanır. Kontrat ID'si ve işlem bağlantıları yayımlanana kadar zincir üstü havuz demosu tamamlandı denmez.
2. Demo anchor arayüzünde her adım **“Simülasyon”** diye işaretlenir ve gerçek TRY, token veya kullanılabilir bakiye oluşmadığı açıklanır. Banka/KYC bilgisi toplanmaz; ödeme talimatı, cüzdan imzası, anchor API çağrısı ve gerçek/test varlığı basımı yapılmaz. Testnet faucet ayrı bir test varlığı edinme yolu olarak etiketlenir.
3. Hackathon DOC'undaki **gerçek TL yatırma/çekme karşılığında kullanılabilir Stellar bakiyesi** şartı bu tasarımla karşılanmaz. Demo sunumunda karşılandı denmez.
4. AI denetimi çalıştırılırsa raporun dayanağı, kural/belge sürümü ve insan kararı gösterilir; ödeme ve sözleşme geçerliliği AI'a bırakılmaz.

## Gerçek para ürünü için ayrı karar

Davetli altın günü koordinasyonu ile halka açık garantili ev/araç tasarruf finansmanı farklı ürünlerdir. Tanıtım ve Testnet deneyi, ikisine de otomatik faaliyet izni vermez. Gerçek fon, TRY anchor, kripto transferi, satıcı ödemesi, ipotek/rehin veya belirli tarihte ev/araç teslimi ancak ürünün somut rolü için hukuk ve yetkili kurum değerlendirmesi yapıldıktan sonra tasarlanır. Lisanslı finansman yolu [ayrı belgede](legal-ai-path.md), altın günü sınırı [burada](altin-gunu-legal-boundary.md) incelenmiştir.

## Teknik dayanak

- [Stellar RPC sağlayıcıları ve Mainnet uç noktası](https://developers.stellar.org/docs/data/apis/rpc/providers)
- [Stellar ağları ve passphrase](https://developers.stellar.org/docs/networks)
- [Vite ortam değişkenleri](https://vite.dev/guide/env-and-mode)
- [Hackathon DOC özeti](hackathon-handbook.md) (yerel kaynak; depo dışı)

Bu belge bir dağıtım/kapsam kaydıdır; hukuki uygunluk görüşü değildir.
