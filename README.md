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


## `Request` `Response` api (actually, builder needed lol)
- create/build with
  - for request: method, target
  - for response: code, optional text
  - for message or builder — transfer and content strategies:
    - TE: content-length, EOF or chunked (default content-length if body non-zero)
    - CE: with compression, and if yes — how (default? default to no compression? or based on content-type?)
  - ### so, mby better:
    - simple — quick, that is with_code or with_code_and_headers or smthn
    - builder — with all that bs above
- then optionally set:
  - http version (default 1.1)
  - logger (ad. earlier idea)
- ### end the builder here?
- push header -> Result, fail if body already began sending
  - single
  - append from slice
  - maybe append from iter (later)
- append body
  - append_body_chunk
- write / send / get the bytes
  - write_available and write_final -> Result, fail if content-length mismatch
  - write_available(impl Write) and write_final(impl Write)
  - so, similarly, to_bytes_available and to_bytes_final?
  - how to handle not-yet-ended preamble? have it be manually "closed"?
    or close it automatically (when?) or just don't close at all? just get the bytes at this point
    and if user adds headers it's not our problem?
  - how to handle mixed write and to_bytes with "flushed_bytes" field?
  - maybe just screw to_bytes entirely?
