# Altın günü modeli ile düzenlenmiş tasarruf finansmanı arasındaki sınır

**Düzeltme (19 Eylül 2026):** Önceki kişisel kasa önerisi, altın günü modelini gereğinden fazla terk ediyordu. 6361 sayılı Kanun, arkadaşların kendi aralarında sırayla katkı yaptığı her grubu açıkça yasaklamıyor. Kanun, belirli koşullarla konut/iş yeri/taşıt edinimi için müşterilere finansman kullandıran ve toplanan tasarrufları yöneten **tasarruf finansman faaliyetini** tanımlıyor. Tasarruf finansman sözleşmesinde şirketin organizasyon ücreti alma hakkı da yer alıyor. Kapalı bir arkadaş grubuyla halka sunulan ticari bir ürünün aynı hukuki statüde olduğu varsayılamaz. Bunun kesin sınırı, ürünün fiili işleyişine göre hukukçu ve ilgili kurumlarla değerlendirilmelidir.

**Güncel uygulama kararı:** [Mainnet tanıtımı](mainnet-showcase.md) işlemsizdir. Testnet hedefi ayrı sponsor olmadan davetli grup ve yalnız mevcut turun iadesidir; anchor demo olarak kalır. Bu karar, aşağıdaki hukuki soruları ortadan kaldırmaz; gerçek fon kabul edilmeden önce ayrıca çözülür.

## Mevcut planda riski doğuran unsurlar

| Unsur | Altın gününden farkı / hukuki soru |
|---|---|
| FuzulEv/FuzulOto gibi **ev/araç edinimi** vaadi ve önceden belirlenmiş tahsisat | 6361 md. 3 ve 39/A'daki konu, amaç ve finansman hakkına yaklaşır. Genel amaçlı arkadaş günüyle aynı değildir. |
| Kullanıcılara sürekli açık platformun grup kurması, takvimi işletmesi ve tahsisatı yönetmesi | Platform yalnızca kayıt aracı mı, yoksa tasarrufları yöneten finansman organizatörü mü? Ücret olmasa da bu rol ayrıca incelenmelidir. |
| Erken teslimat almış kişinin sonraki katkıyı ödememesi | Diğer üyelerin önceki turlarda ödedikleri tutarlar satıcıya gitmiştir. Ayrı kurumsal/teminat yapısı olmadan tam iade veya teslimat garantisi verilemez. |
| Soroban kontratında ortak fonun tutulması ve **satıcıya doğrudan token aktarımı** | BDDK kapsamına girerse düzenlenmiş TL fon havuzunun yerini USDC/Soroban tutmaz. Satıcıya kripto varlıkla doğrudan/dolaylı ödeme için TCMB kuralı ayrıca değerlendirilmelidir. |
| Platformun TL ↔ token dönüşümü, kripto transferi veya anahtar/saklama hizmeti sunması | BDDK dışında SPK ve ödeme mevzuatı gündeme gelir. Anchor kullanılması platforma kendiliğinden izin vermez. |

Bu unsurlardan **hiçbiri tek başına** “bu proje kesin yasa dışı” sonucunu vermez. Özellikle mevcut Testnet prototipi gerçek müşteri fonu toplamaz. Risk, bunların gerçek parayla ve halka sunulan sürekli bir işletme olarak birleşmesindedir. Mevcut planda organizasyon ücreti tanımlanmış değildir; bu, ticari hizmetin niteliğinin otomatik olarak serbest olduğu anlamına da gelmez.

## Kura ve büyük grup eki (19 Eylül 2026, akşam)

Ürün yönü Fuzul Ev/Oto mantığına yaklaştırıldı: sıra yerine **kura** ve **30 üyeye kadar** grup (yalnızca Testnet, gerçek para ve teslimat yok). Bu iki unsur yukarıdaki riskleri artırır:

| Unsur | Ek soru |
|---|---|
| Çekilişle alıcı belirlenmesi | Ödeme yapanlar arasında şansa bağlı tahsisat, tasarruf finansmanındaki çekiliş mekanizmasına ve şans oyunları mevzuatına yaklaşabilir. Hangi çerçeveye girdiği hukukçuyla değerlendirilmelidir. |
| Daha büyük ve daha az tanıdık grup | "Tanıdık, kapalı arkadaş grubu" niteliği zayıflar; platformun grubu kurup işletmesi finansman organizatörü rolüne yaklaşır. |
| Rastgelelik | Zincir üstü PRNG hackathon düzeyindedir; gerçek fonla adil çekiliş iddiası taşımaz. |
| Ekonomik açık | Büyük grupta tur tutarı ve erken teslim alanın temerrüdünden doğan açık büyür; kontrat bunu kapatmaz. |

Bu nedenle arayüz ve belgeler kura/büyük grubu **Testnet simülasyonu** olarak etiketler, ev/araç teslimi veya garantisi vaat etmez. Gerçek para akışı için önce lisanslı yapı ve hukuki değerlendirme gerekir.

## Altın günü özünü koruyan daha dar ürün

- Tanıdık kişilerin kendi oluşturduğu, davetli ve sabit üyeli bir grup; herkese açık eşleştirme veya platform tarafından teslimat vaadi yok.
- Genel amaçlı sıralı katkı ve teslim; ev/araç **finansmanı**, satıcı doğrulaması, tapu/rehin ve erken mal teslimi ürün vaadi yapılmaz.
- Grup üyeleri katkı, sıra, gecikme ve ayrılma koşullarını birbirleriyle kabul eder. Uygulama operatörünün sıra değiştirme, fon çekme veya kendi adına tahsisat yapma yetkisi olmaz.
- Platform adına organizasyon ücreti, başkası adına avans veya borç tahsilatı tasarlanmaz. Bir katılımcının gecikmesi diğer katılımcılar arasında sözleşme ve genel hukuk meselesi olarak kalır; kod mülke el koyamaz veya tahsilatı garanti edemez.
- Gerçek para akışı için kullanıcı fonunun kimde tutulacağı ve transferi kimin sunacağı ayrıca belirlenir. Soroban'da USDC tutulması, kapalı grup olsa bile SPK/TCMB sorularını kendiliğinden çözmez. En düşük iddialı sürüm, üyelerin kendi yetkili hesapları arasındaki TL transferlerini kayıt/kanıt ile eşleştiren koordinasyon yazılımıdır; burada dahi ödeme emri ve veri işleme rolü incelenir.

Bu tasarım **lisans muafiyeti garantisi değildir**. Özellikle halka açık, ücretli ve ev/araç tahsisatı vaat eden sürüm için BDDK görüşü veya lisanslı şirketle yapı gerekir. Kapalı arkadaş grubunun sözleşmesi, vergisi, uyuşmazlık çözümü ve kullanılan ödeme/kripto altyapısı ayrıca değerlendirilmelidir.

## Hackathon DOC gerilimi

[Hackathon DOC'u](hackathon-handbook.md) gerçek TL giriş/çıkışı ile kullanılabilir Stellar bakiyesi istiyor. Bu şart, yalnızca TL transferlerini kaydeden altın günü uygulamasıyla karşılanmaz. Gerçek TRY anchor/yerel ödeme ortağı henüz doğrulanmadı. Bir sağlayıcı bulunursa işlem izinleri ve platform rolü incelenmeden gerçek parayı kontrat havuzuna yönlendirmemeliyiz. Testnet tokenı, demo banka ekranı veya AI raporu bu şartın yerine geçmez. DOC uyumu ile gerçek Türkiye ürünü için hukuki uygunluk ayrı kanıtlanmalıdır.

## Önerilen iki ürün yolu

1. **Altın günü ürünü:** Davetli ve sabit bir grup kendi sırasını ve katkısını belirler; uygulama onay, kayıt, bildirim ve açık hesap sunar. Ev/araç gruptaki kişilerin kendi amacı olabilir, ancak uygulama mülk edinimi, tahsisat tarihi veya eksik ödemeyi karşılama garantisi satmaz. Gerçek TL/kripto transferini kimin sunduğu ayrı hukuki analize tabi tutulur. Bu yol başlangıç fikrini korur ama otomatik bir lisans muafiyeti değildir.
2. **Garantili ev/araç finansmanı ürünü:** BDDK faaliyet izni olan tasarruf finansman şirketi müşteri sözleşmesinin tarafı, ayrılmış TL fonunun yöneticisi, tahsisatın ve finansal yükümlülüklerin sorumlusu olur. Stellerpool, izin verilen dış hizmet sınırları içinde izleme, belge bütünlüğü ve insan denetçiye yardımcı AI sunar. Satıcı ödemesi ve mülkiyet/teminat işlemleri yetkili fiat/hukuk kanallarında kalır. Bir startupın kendi başına aynı garantiyi sunması ayrı faaliyet izni, sermaye ve işletme yükümlülükleri gerektirir. Lisanslı şirket olması kamusal mevduat sigortası anlamına gelmez.

**Garanti sınırı:** Sponsorsuz Testnet planı yalnızca henüz satıcıya ödenmemiş turun katkısını kontrattan iade etmeyi hedefler. Önceki turdaki ödeme geri çağrılamaz; eksik gelecek taksitler, ev/araç teslimi, mülkiyet devri ve TL satın alma gücü garanti edilmez. Gerçek teslimat taahhüdü lisanslı tarafın likidite, sözleşme, tahsilat ve hukuki teminat süreçleriyle değerlendirilir. BDDK'ya göre tasarruf finansman müşterilerinin tasarrufları kamusal mevduat sigortası kapsamında değildir.

**Anchor seçim kapısı:** Sağlayıcının gerçek TRY yatırma/çekmeyi, Stellar Mainnet varlığını, kullanılabilir bakiyeyi ve kullanıcı kimlik doğrulamasını desteklediği; Türkiye'deki rol ve izinleri; ihraççı, geri ödeme, ücret ve limitleri yazılı olarak doğrulanır. SEP-6/SEP-24 teknik uygunluğu tek başına yetmez. BiLira'nın kamuya açık TRYB ağ listesinde Stellar görünmüyor; Bridge/BlindPay'in DOC'ta adlarının geçmesi de canlı TRY koridorunu doğrulamaz. Sağlayıcı doğrulanmadan hackathon'un gerçek TL şartı tamamlandı denmez. Anchor, tasarruf finansmanı faaliyet izni veya ev/araç teslimat garantisi vermez.

## Kaynaklar

- [7292 sayılı Kanun ile 6361'e eklenen tanımlar ve md. 39/A–39/B (TBMM)](https://cdn.tbmm.gov.tr/KKBSPublicFile/D27/Y4/T2/KanunMetni/cce61925-4314-4327-8080-08a7df802e00.html)
- [BDDK tasarruf finansmanı SSS](https://www.bddk.org.tr/Sss/Liste/117)
- [SPK kripto varlık hizmet sağlayıcı duyurusu](https://spk.gov.tr/duyurular/basin-duyurulari/2024/kripto-varlik-hizmet-saglayicilara-iliskin-duyuru_02072024)
- [TCMB ödemelerde kripto varlık yönetmeliği](https://www.tcmb.gov.tr/wps/wcm/connect/6937855a-7c29-4d08-a26e-51ef3273c022/%C3%96demelerde+Kripto+Varl%C4%B1klar%C4%B1n+Kullan%C4%B1lmamas%C4%B1na+Dair+Y%C3%B6netmelik.pdf?MOD=AJPERES)
- [TCMB ödeme iş modelleri rehberi](https://www.tcmb.gov.tr/wps/wcm/connect/5b814310-a57b-4e64-82fe-76e520d0e0b1/%C3%96demeler+Alan%C4%B1nda+Sunulan+%C4%B0%C5%9F+Modellerine+%C4%B0li%C5%9Fkin+Rehber.pdf?MOD=AJPERES)
- [Stellar anchor ve SEP-24 açıklaması](https://developers.stellar.org/docs/build/apps/example-application-tutorial/anchor-integration)
- [BiLira TRYB desteklenen ağlar](https://www.bilira.co/tr/tryb-kullanimi)
