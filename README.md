# Auto Get USTB Ammeter 自动获取北科电表电量

## 简介

用来自动获取北科电表电量，并整理成 csv 表格。知道电表号就可以了，什么，你不知道电表号在哪？就在走廊电表箱打开的中间。

电表数据每日0点更新，所以每天看一次就行，也可以把打包出来的软件设置为电脑定时任务。

> 然后现在主分支现在的版本是我尝试使用 iced 库写了一个图形页面，好事的人可以看看🤔，直接 cargo run 就行。

## 使用方法

终端中把 auto-get-ustb-ammeter 拖进去，按回车运行。

然后输入电表号（一般是8位数字），按下回车，会生成一个文件（不要改这个文件的格式，可以用 Excel 打开）。\
第一列数据第一行是电表号，下次再使用本程序就会自动读取，第二列是日期，第三列是剩余电量，第四列是据上一次测试平均每天耗电情况。  

例如：

ammeter_data.csv: 
```csv
11010000,Date,Remain(KWh),Average everyday usage since last date
,2024-10-27,231,0
```
