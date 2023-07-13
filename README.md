# 貨幣匯率轉換程式

這是一個可以在任何聊天框裡使用的貨幣匯率轉換程式。只需要輸入 "/ex 100USD=TWD" 或 "/ex 100 USD TWD"，就可以將 100 美金轉換成新台幣。


## 如何使用

1. 在聊天框輸入 "/ex"。
2. 在 "/ex" 後面輸入數字，並加上要轉換的貨幣類型，例如 "100USD=TWD" 或 "100 USD TWD"。
3. 程式會自動回傳轉換後的金額。


## 範例

* 輸入 "/ex 100USD=TWD"，回傳 "匯率轉換結果： 3000 新台幣"。
* 輸入 "/ex 100USD TWD"，回傳 "匯率轉換結果： 3000 新台幣"。
* 輸入 "/ex 50EUR=USD"，回傳 "匯率轉換結果： 60 美金"。
* 輸入 "/ex 50EUR USD"，回傳 "匯率轉換結果： 60 美金"。


## 貨幣支援列表

程式支援常見法幣，例如：

* 美金 (USD)
* 歐元 (EUR)
* 日圓 (JPY)
* 英鎊 (GBP)
* 港幣 (HKD)
* 澳幣 (AUD)
* 新台幣 (TWD)


## 注意事項

* 目前匯率數據是從開放資料平台取得，僅供參考，實際匯率以銀行公告為準。
* 輸入格式必須為 "{數字?}{貨幣類型}={貨幣類型}" 或 "{數字?}{貨幣類型} {貨幣類型}"，且貨幣類型必須為支援列表中的其中一種。

**本說明由 ChatGPT 自動產生**
