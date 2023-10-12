# azure-acr cli-rs

## 配置文件说明
>
> 命名为： config.toml

样例文件为 `config.sample.toml`

```toml
[azure]
# azure tenant id
tenant_id = "xxxxxxxxxxxxxxx"

[acr]
# access app privilege: delete 
# acr access app's id
image_manager_id = "xxxxxxxxxxx"
# acr access app's password
image_manager_pwd = "xxxxxxxxx"
# acr private name  like "james.azurecr.io"
endpoint = "xxxxxx.azurecr.io"

# @type: array
# image name filter 
# keyword: keep the image_name which contains the keyword
[[filter.image_name.keep.rules]]
keyword = "/"
[[filter.image_name.keep.rules]]
keyword = "-"
# tag filter

# 1. tag won't be deleted which contains the filter keyword 
# 2. the rest tags order by create time desc
# 3. keep top 'default.num' of the tags processed in step 2, finally the rest tags will be deleted
[filter.tag.keep]
default.num = 20

# @type: array
# tag name filter 
# keyword: keep the tag which contains this keyword
[[filter.tag.keep.rules]]
keyword = "stable"
[[filter.tag.keep.rules]]
keyword = "latest"
```
