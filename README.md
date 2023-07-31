# crossgate

## build

```
docker buildx build \
  --platform linux/arm/v7,linux/arm64/v8,linux/386,linux/amd64,linux/ppc64le \
  -t stream:0.1\
  -f Dockerfile.stream \
  .
```


# Condition使用方法
## 使用说明

```rust
// 1. validates时可直接携带key值
format!("name = '123'");
// 2. condition时不可携带key值
format!("{} = '123'", item.name); 
```


## 基本类型

- string
    
    ```rust
    // 1. 等价比较
    format!("name = '{}'", v); // 字符串一定要包裹在 '' 中
    // 2. 不等于
    format!("name != '{}'", v); // 字符串不可为 xxx
    // 3. 长度
    format!("len(name) > 5");
    // 4. regex(like)
    format!("name ! '{}'", "abc"); // 包含abc
    // 5. not_like
    format!("name !! '{}'", "abc");// 不包含abc
    // 6. in
    format!("name ~ ('1','2','3','4')"); //  name在 1,2,3,4这几个中
    ```
    
- number
    
    ```rust
    // 1. 等于
    format!("a = 1"); 
    // 2. 不等于
    format!("a != 1"); 
    // 3. 大于
    format!("a > 1"); 
    // 4. 大于或等于
    format!("a >= 1"); 
    // 5. in
    format!("a ~~ (1,2,3)"); 
    // 6. not_in
    format!("a ~~ (1,2,3)");
    // 7. belong
    format!("a << (1,2,3)");
    ```
    
- null
    
    ```rust
    // 1. is_null
    format!("a ^ null"); //a is null
    // 2.  is_not_null
    format!("a ^^ null"); //a is not null
    ```
    
- bool
    
    ```rust
    // 1. is_true
    format!("a = true");
    // 2. is_not_true
    format!("a != true");
    ```
    

## 组合类型

```rust
// 1. 与
format!("a = true && a = false");
// 2. 或
format!("a = true || a = false");
// 3. 组合
format!("a = true || (a = false && a = true)");
```
