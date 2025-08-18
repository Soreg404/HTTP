http://http.badssl.com/
http://httpforever.com/


API ideas
## 0
`Request`, `Response`: build and send
`PartialRequest`, `PartialResponse`: read/collect; can't be used on their own - 
require transforming into `Request`  /`Response`

## 3
Partials cover only collecting an HTTP message;
Primaries hold only raw headers, body and variant specific fields, i.e. method for request

## 1
- request/response builder
- req/res default

## 2
req/res -> multipart req/res


## collecting http message
preamble (that is, first line and headers) + body;
  where body can be subject to Transfer Encoding and Content Encoding
  body is either a single file or is "multipart"

```
stream                                          
  |                                          
collect Preamble                                          
  |                                          
check Transfer Encoding                                          
                     |                                          
      is Transfer-Encoding header present?                                          
  /                                          \                                          
yes                                          no                                          
|                                             |                                          
(!) currently, only supported               is Content-Length header present?           
value is "chunked"                             /                      \                
otherwise @Fail                               yes                     no                
      |                                       |                        |           
  take chunks                        take exact length              take until 
  until NULL chunk                     of the body                 signal_connection_close              
                                                                                    
                                                                                    
                                                                                    
                                                                                    
                                                                                    
  |
is multipart?
```

Transfer-Encoding
|
Content-Encoding
|
optional Multipart


## message buffers, in order
- master_buffer (pre-transfer-decoded)
- chunks_ranges_vector
- optional decompress intermediate buffer
- complete_body_buffer (post-content-decoding)
- attachments_boundaries_vector


## needed message collectors metadata
- optional content length
- optional transfer encoding strategy
- optional content encoding strategy
- optional multipart boundary


## idea: enum for transfer strategy
with fields:
- none or til EOF - when to fail?
  - if none, then no body, as in empty request/response?
  - what to do with "connection: close" and HTTP/1.0?
- content-length
- chunked

## behavior on duplicate headers - keep last

## idea: with_verbose_output(impl Write, level)


## builder for `Request` and `Response` is needed, and Multiparts are a separate thing
## Nah, I take that back, dedicated builder is not needed, `with_code` or `with_headers` will (probably) do
## or, an idea for later, first build the Preamble, then convert to "only append to body, headers readonly" thing
