Various notes I took to understand this topic better : 

## CSS FlexBox 

It is (one of) the "best current" models to handle layout & responsiveness 

There are alternatives : fixed layout (like grid-based), which is generally horrible, some others are constraint-based and make use of fast constraint solvers like Cassowary ; this was used by Apple in the past but there's no reference lib. 

In the flexbox model, most dimensions are expressed relatively to the dimensions of the parent container (the top container being the window). But I can also give FlexBox absolute units. 

## Absolute units 

In CSS, a pixel (px) is defined as a 96 DPI pixel and is not so absolute. So it'll be sized accordingly to the system settings. 