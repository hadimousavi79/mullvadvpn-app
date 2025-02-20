import React from 'react';
import styled, { PolymorphicComponentProps, WebTarget } from 'styled-components';

import { Colors, Typography, typography, TypographyProperties } from '../../foundations';
import { TransientProps } from '../../types';

export type Test = React.PropsWithChildren<{
  variant?: Typography;
  color?: Colors;
}>;

export type TextProps<T extends WebTarget> = PolymorphicComponentProps<'web', Test, T, T>;

const StyledText = styled.span<TransientProps<TypographyProperties>>((props) => ({
  color: 'var(--color)',
  fontFamily: props.$fontFamily,
  fontWeight: props.$fontWeight,
  fontSize: props.$fontSize,
  lineHeight: props.$lineHeight,
}));

export const Text = <T extends WebTarget>({
  variant = 'bodySmall',
  color = Colors.white,
  children,
  style,
  ...props
}: TextProps<T>) => {
  const { fontFamily, fontSize, fontWeight, lineHeight } = typography[variant];
  return (
    <StyledText
      style={
        {
          '--color': color,
          ...style,
        } as React.CSSProperties
      }
      $fontFamily={fontFamily}
      $fontWeight={fontWeight}
      $fontSize={fontSize}
      $lineHeight={lineHeight}
      {...props}>
      {children}
    </StyledText>
  );
};

Text.displayName = 'Text';
